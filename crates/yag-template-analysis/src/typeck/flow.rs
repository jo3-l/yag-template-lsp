use std::mem;

use bitflags::bitflags;
use ecow::EcoString;
use rustc_hash::{FxHashMap, FxHashSet};

use super::ty::{union, Ty};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockKind {
    /// The body of an `range` or `while` action.
    LoopBody,
    /// The `try` block within a try-catch action.
    TryBody,
    /// All other block types.
    Other,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct FlowFacts: u8 {
        /// Whether a return action occurs along at least one control flow paths through the block.
        /// The analysis is necessarily conservative by Rice's theorem — that is, there may be some
        /// programs that never return from a given block yet do have this flag set — as is the case
        /// for all subsequent flags.
        const HAS_POTENTIAL_RETURN = 1 << 0;
        /// Whether a return action occurs along all control flow paths through the block.
        const HAS_DEFINITE_RETURN = 1 << 1;

        /// Whether a fallible function call occurs along at least one control flow path through the block.
        const HAS_FALLIBLE_FN_CALL = 1 << 2;

        /// Whether a loop break action occurs along at least one control flow path through the block.
        const HAS_POTENTIAL_LOOP_BREAK = 1 << 3;
        /// Whether a loop break action occurs along all control flow paths through the block. If this flag
        /// is set, `HAS_POTENTIAL_LOOP_BREAK` will also be set.
        const HAS_DEFINITE_LOOP_BREAK = 1 << 4;
        /// Whether a loop continue action occurs along at least one control flow path through the block.
        const HAS_POTENTIAL_LOOP_CONTINUE = 1 << 5;
        /// Whether a loop continue action occurs along all control flow paths through the block. If this flag
        /// is set, `HAS_POTENTIAL_LOOP_CONTINUE` will also be set.
        const HAS_DEFINITE_LOOP_CONTINUE = 1 << 6;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VarAssignInfo {
    /// The union of all potential types for this variable, assuming that a control flow path
    /// through an assignment was taken.
    pub(crate) ty: Ty,
    /// Whether an assignment to this variable occurs along all control flow paths through the block.
    pub(crate) is_definite: bool,
}

#[derive(Debug)]
pub(crate) struct Block {
    pub(crate) kind: BlockKind,
    pub(crate) initial_context_ty: Ty,
    pub(crate) flow_facts: FlowFacts,
    /// The union of all possible types returned from this block, or `Ty::Never` if there are no
    /// return actions in the block.
    pub(crate) return_ty: Ty,
    /// The union of all possible error types thrown from function calls in the block, or
    /// `Ty::Never` if no fallible function calls are present.
    pub(crate) throw_ty: Ty,
    pub(crate) declared_vars: FxHashSet<EcoString>,
    pub(crate) var_assigns: FxHashMap<EcoString, VarAssignInfo>,
    /// Types of variables that are declared or potentially assigned to within the block. When
    /// looking up variable types, it is still necessary to scan parent blocks for inherited
    /// variables.
    pub(crate) resolved_var_types: FxHashMap<EcoString, Ty>,
}

impl Block {
    pub(crate) fn new(kind: BlockKind, initial_context_ty: Ty) -> Self {
        Self {
            kind,
            initial_context_ty,
            flow_facts: FlowFacts::empty(),
            return_ty: Ty::Never,
            throw_ty: Ty::Never,
            declared_vars: FxHashSet::default(),
            var_assigns: FxHashMap::default(),
            resolved_var_types: FxHashMap::default(),
        }
    }

    // Merge an inner block into the current block.
    pub(crate) fn merge_inner(&mut self, mut child: Block) {
        self.flow_facts |= child.propagate_facts();
        self.return_ty = union(&self.return_ty, &child.return_ty);
        self.throw_ty = union(&self.throw_ty, child.propagate_throw_ty());
        self.commit_inner_assignments(child.propagate_var_assigns());
    }

    /// Merge two child blocks, of which one and only one is executed in a given program execution,
    /// into the current block.
    pub(crate) fn merge_inner_branches(&mut self, mut left: Block, mut right: Block) {
        const DEFINITE_FACTS: FlowFacts = FlowFacts::HAS_DEFINITE_RETURN
            .union(FlowFacts::HAS_DEFINITE_LOOP_BREAK)
            .union(FlowFacts::HAS_DEFINITE_LOOP_CONTINUE);

        let left_facts = left.propagate_facts();
        let right_facts = right.propagate_facts();
        // Definite facts (e.g., HAS_DEFINITE_RETURN) that assert properties about all control flow
        // paths must be set in both branches.
        self.flow_facts |= DEFINITE_FACTS & (left_facts & right_facts);
        // Other 'uncertain' facts (e.g., HAS_POTENTIAL_RETURN) need only be set in one branch.
        self.flow_facts |= !DEFINITE_FACTS & (left_facts | right_facts);

        self.return_ty = union(&self.return_ty, &left.return_ty);
        self.return_ty = union(&self.return_ty, &right.return_ty);

        self.throw_ty = union(&self.throw_ty, left.propagate_throw_ty());
        self.throw_ty = union(&self.throw_ty, right.propagate_throw_ty());

        // Merge variable assignments from the right child into the left child.
        let mut merged_var_assigns = left.propagate_var_assigns();
        for (var, assign) in right.propagate_var_assigns() {
            merged_var_assigns
                .entry(var)
                .and_modify(|merged_assign| {
                    merged_assign.ty = union(&merged_assign.ty, &assign.ty);
                    // The variable assignment must occur along all code paths in both
                    // branches to be definite.
                    merged_assign.is_definite &= assign.is_definite;
                })
                .or_insert(VarAssignInfo {
                    ty: assign.ty,
                    // The variable assignment only occurs in one branch, so is not definite.
                    is_definite: false,
                });
        }
        self.commit_inner_assignments(merged_var_assigns)
    }

    pub(crate) fn commit_inner_assignments(
        &mut self,
        var_assigns: impl IntoIterator<Item = (EcoString, VarAssignInfo)>,
    ) {
        for (var, assign) in var_assigns {
            self.var_assigns
                .entry(var.clone())
                .and_modify(|existing_assign| existing_assign.ty = union(&existing_assign.ty, &assign.ty))
                .or_insert_with(|| assign.clone());

            if assign.is_definite {
                self.resolved_var_types.insert(var, assign.ty);
            } else {
                self.resolved_var_types
                    .entry(var)
                    .and_modify(|cur_ty| *cur_ty = union(&cur_ty, &assign.ty))
                    .or_insert(assign.ty);
            }
        }
    }

    /// Extract the variable assignments that propagate into the parent block, consuming
    /// `self.var_assigns`.
    pub(crate) fn propagate_var_assigns(&mut self) -> FxHashMap<EcoString, VarAssignInfo> {
        let mut var_assigns = mem::take(&mut self.var_assigns);
        var_assigns.retain(|var, _| !self.declared_vars.contains(var));
        var_assigns
    }

    /// Extract the throw type that propagates into the parent block.
    pub(crate) fn propagate_throw_ty(&self) -> &Ty {
        if self.kind == BlockKind::TryBody {
            &Ty::Never
        } else {
            &self.throw_ty
        }
    }

    /// Extract the flow facts that propagate into the parent block.
    pub(crate) fn propagate_facts(&self) -> FlowFacts {
        match self.kind {
            BlockKind::LoopBody => self.flow_facts.difference(
                FlowFacts::HAS_POTENTIAL_LOOP_BREAK
                    | FlowFacts::HAS_DEFINITE_LOOP_BREAK
                    | FlowFacts::HAS_POTENTIAL_LOOP_CONTINUE
                    | FlowFacts::HAS_DEFINITE_LOOP_CONTINUE,
            ),
            BlockKind::TryBody => self.flow_facts.difference(FlowFacts::HAS_FALLIBLE_FN_CALL),
            BlockKind::Other => self.flow_facts,
        }
    }

    /// Could a `return`, fallible function call, `break`, or `continue` potentially have occurred
    /// before this point in the block?
    pub(crate) fn has_potential_jmp(&self) -> bool {
        self.flow_facts.intersects(
            FlowFacts::HAS_POTENTIAL_RETURN
                | FlowFacts::HAS_FALLIBLE_FN_CALL
                | FlowFacts::HAS_POTENTIAL_LOOP_BREAK
                | FlowFacts::HAS_POTENTIAL_LOOP_CONTINUE,
        )
    }

    /// Has a `return`, `break`, or `continue` definitely occurred before this point in the block?
    /// (If so, any code in the block past this point is unreachable.)
    pub(crate) fn has_definite_jmp(&self) -> bool {
        self.flow_facts.intersects(
            FlowFacts::HAS_DEFINITE_RETURN | FlowFacts::HAS_DEFINITE_LOOP_BREAK | FlowFacts::HAS_DEFINITE_LOOP_CONTINUE,
        )
    }
}
