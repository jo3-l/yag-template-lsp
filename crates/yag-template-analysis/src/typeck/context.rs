use ecow::EcoString;
use rustc_hash::FxHashMap;
use yag_template_syntax::ast;

use super::block::Block;
use super::output::TypeckOutput;
use super::ty::foreign::TypeDefinitions;
use super::ty::Ty;

#[derive(Debug)]
pub(crate) struct TypeckContext<'e> {
    pub(crate) env: &'e TypeDefinitions,
    pub(crate) is_main_template: bool,
    pub(crate) cur_block: Block,
    pub(crate) parent_blocks: Vec<Block>,
    pub(crate) call_stack: Vec<EcoString>,
    pub(crate) assoc_templates: FxHashMap<String, AssocTemplate>,
    pub(crate) out: Option<TypeckOutput>,
}

#[derive(Debug)]
pub(crate) struct AssocTemplate {
    actions: ast::ActionList,
    erased: bool,
    instantiations: FxHashMap<Ty, Ty>,
}
