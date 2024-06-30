use std::mem;

use ecow::EcoString;
use rustc_hash::FxHashMap;
use yag_template_syntax::ast;

use super::flow::{Block, BlockKind, VarAssignInfo};
use super::output::TypeckOutput;
use super::ty::foreign::TypeDefinitions;
use super::ty::{union, Ty};

#[derive(Debug)]
pub(crate) struct TypeckContext<'e> {
    pub(crate) env: &'e TypeDefinitions,
    pub(crate) cur_block: Block,
    pub(crate) parent_blocks: Vec<Block>,
    pub(crate) call_stack: Vec<EcoString>,
    pub(crate) assoc_templates: FxHashMap<String, AssocTemplate>,
    pub(crate) out: Option<TypeckOutput>,
}

impl TypeckContext<'_> {
    pub(crate) fn enter_block(&mut self, kind: BlockKind, new_context_ty: Ty) {
        let new_block = Block::new(&self.cur_block, kind, new_context_ty);
        self.parent_blocks.push(mem::replace(&mut self.cur_block, new_block));
    }

    pub(crate) fn inherit_context_ty(&self) -> Ty {
        self.cur_block.context_ty.clone()
    }

    pub(crate) fn exit_block(&mut self) -> Block {
        mem::replace(
            &mut self.cur_block,
            self.parent_blocks
                .pop()
                .expect("should only call exit_block() when parent is present"),
        )
    }

    pub(crate) fn assign(&mut self, var: &str, ty: Ty) {
        let exists = self.cur_block.declared_vars.contains(var)
            || self.parent_blocks.iter().any(|block| block.declared_vars.contains(var));
        if !exists {
            // TODO: issue error
        }

        if self.cur_block.potentially_jumps() {
            self.cur_block
                .var_assigns
                .entry(var.into())
                .and_modify(|existing_assign| existing_assign.ty = union(&existing_assign.ty, &ty))
                .or_insert_with(|| VarAssignInfo {
                    ty: ty.clone(),
                    is_definite: false,
                });
        } else {
            self.cur_block.var_assigns.insert(
                var.into(),
                VarAssignInfo {
                    ty: ty.clone(),
                    is_definite: true,
                },
            );
        }
        self.cur_block.scoped_var_types.insert(var.into(), ty);
    }

    pub(crate) fn declare(&mut self, var: &str, ty: Ty) {
        self.cur_block.declared_vars.insert(var.into());
        self.cur_block.scoped_var_types.insert(var.into(), ty);
    }
}

#[derive(Debug)]
pub(crate) struct AssocTemplate {
    pub(crate) actions: ast::ActionList,
    pub(crate) overflowed_instantiations: bool,
    pub(crate) instantiations: FxHashMap<Ty, Ty>,
}
