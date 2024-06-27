use bitflags::bitflags;
use ecow::EcoString;
use rustc_hash::{FxHashMap, FxHashSet};

use super::ty::Ty;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BlockKind {
    /// A `range` or `while` loop block.
    Loop,
    /// An `if` or `with` conditional block.
    Conditional,
    /// All other block types.
    Other,
}

#[derive(Debug)]
pub(crate) struct Block {
    pub(crate) kind: BlockKind,
    pub(crate) initial_context_ty: Ty,
    pub(crate) flow_flags: FlowFlags,
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
            flow_flags: FlowFlags::default(),
            return_ty: Ty::Never,
            throw_ty: Ty::Never,
            declared_vars: FxHashSet::default(),
            var_assigns: FxHashMap::default(),
            resolved_var_types: FxHashMap::default(),
        }
    }
}

bitflags! {
    #[derive(Debug, Default)]
    pub(crate) struct FlowFlags: u8 {
        /// Whether a return action occurs along at least one control flow paths through the block.
        /// The analysis is necessarily conservative by Rice's theorem — that is, there may be some
        /// programs that never return from a given block yet do have this flag set — as is the case
        /// for all subsequent flags.
        const HAS_POTENTIAL_RETURN = 1 << 0;
        /// Whether a return action occurs along all control flow paths through the block.
        const HAS_DEFINITE_RETURN = 1 << 1;

        /// Whether a fallible function call occurs along at least one control flow path through the block.
        const HAS_POTENTIAL_THROW = 1 << 2;

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

#[derive(Debug)]
pub(crate) struct VarAssignInfo {
    /// The union of all potential types for this variable, assuming that a control flow path
    /// through an assignment was taken.
    pub(crate) ty: Ty,
    /// Whether an assignment to this variable occurs along all control flow paths through the block.
    pub(crate) is_definite: bool,
}
