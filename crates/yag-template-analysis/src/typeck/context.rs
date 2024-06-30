use std::mem;

use ecow::{EcoString, EcoVec};
use rustc_hash::FxHashMap;
use yag_template_syntax::{ast, TextRange};

use super::flow::{Block, BlockKind, VarAssignInfo};
use super::output::TypeckOutput;
use super::ty::foreign::TypeDefinitions;
use super::ty::{union, Ty};

#[derive(Debug)]
pub(crate) struct TypeckOptions {
    pub(crate) record_output: bool,
    pub(crate) evaluate_new_instantiations: bool,
}

impl Default for TypeckOptions {
    fn default() -> Self {
        Self {
            record_output: true,
            evaluate_new_instantiations: true,
        }
    }
}

#[derive(Debug)]
pub(crate) struct TypeckContext<'e> {
    pub(crate) opts: TypeckOptions,
    pub(crate) env: &'e TypeDefinitions,

    pub(crate) template_name: EcoString,
    pub(crate) call_stack: Vec<EcoString>,

    pub(crate) top_block: Block,
    pub(crate) parent_blocks: Vec<Block>,

    pub(crate) assoc_templates: FxHashMap<EcoString, AssocTemplate>,
    pub(crate) out: TypeckOutput,
}

const ROOT_TEMPLATE_NAME: &str = "<root template>";

impl<'e> TypeckContext<'e> {
    pub(crate) fn new(opts: TypeckOptions, env: &'e TypeDefinitions) -> TypeckContext<'e> {
        Self {
            opts,
            env,
            template_name: ROOT_TEMPLATE_NAME.into(),
            call_stack: Vec::new(),
            top_block: Block::new_detached(BlockKind::default(), env.initial_context_ty.clone()),
            parent_blocks: Vec::new(),
            assoc_templates: FxHashMap::default(),
            out: TypeckOutput::default(),
        }
    }
}

impl TypeckContext<'_> {
    pub(crate) fn enter_block(&mut self, kind: BlockKind, new_context_ty: Ty) {
        let new_block = Block::new(&self.top_block, kind, new_context_ty);
        self.parent_blocks.push(mem::replace(&mut self.top_block, new_block));
    }

    pub(crate) fn inherit_context_ty(&self) -> Ty {
        self.top_block.context_ty.clone()
    }

    pub(crate) fn exit_block(&mut self) -> Block {
        mem::replace(
            &mut self.top_block,
            self.parent_blocks
                .pop()
                .expect("should only call exit_block() when parent is present"),
        )
    }
}

impl TypeckContext<'_> {
    pub(crate) fn lookup(&mut self, var: &str) -> Option<&Ty> {
        self.top_block.scoped_var_types.get(var).or_else(|| {
            self.parent_blocks
                .iter()
                .rev()
                .find_map(|block| block.scoped_var_types.get(var))
        })
    }

    pub(crate) fn assign(&mut self, var: &str, ty: Ty) {
        let exists = self.top_block.declared_vars.contains(var)
            || self.parent_blocks.iter().any(|block| block.declared_vars.contains(var));
        if !exists {
            // TODO: issue error
        }

        if self.top_block.potentially_jumps() {
            self.top_block
                .var_assigns
                .entry(var.into())
                // The previous variable assignment might be along a different control flow path; we
                // mustn't overwrite it.
                .and_modify(|existing_assign| existing_assign.ty = union(&existing_assign.ty, &ty))
                .or_insert_with(|| VarAssignInfo {
                    ty: ty.clone(),
                    is_definite: false,
                });
        } else {
            self.top_block.var_assigns.insert(
                var.into(),
                VarAssignInfo {
                    ty: ty.clone(),
                    is_definite: true,
                },
            );
        }

        // `scoped_var_types` stores live variable types at the current point of analysis, so we can
        // unconditionally overwrite the type regardless of previous assignments to the same
        // variable along different control flow paths. (In contrast, the type stored in
        // `var_assigns` reflects the union of types observable by an observer outside the block, so
        // more care is necessary there.)
        self.top_block.scoped_var_types.insert(var.into(), ty);
    }

    pub(crate) fn declare(&mut self, var: &str, ty: Ty) {
        self.top_block.declared_vars.insert(var.into());
        self.top_block.scoped_var_types.insert(var.into(), ty);
    }
}

#[derive(Debug)]
pub(crate) struct AssocTemplate {
    pub(crate) name: EcoString,
    pub(crate) defn_range: TextRange,
    pub(crate) body: ast::ActionList,
    pub(crate) overflowed_instantiation_cache: bool,
    pub(crate) cached_instantiations: FxHashMap<Ty, Ty>,
}

impl AssocTemplate {
    pub(crate) const MAX_UNIQUE_INSTANTIATIONS: usize = 5;

    pub(crate) fn new(name: EcoString, defn_range: TextRange, body: ast::ActionList) -> Self {
        Self {
            name,
            defn_range,
            body,
            overflowed_instantiation_cache: false,
            cached_instantiations: FxHashMap::default(),
        }
    }
}
