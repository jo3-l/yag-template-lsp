use ecow::EcoString;
use rustc_hash::FxHashMap;
use yag_template_syntax::{SyntaxNodePtr, TextRange};

use super::ty::foreign::FieldOrMethod;
use super::ty::Ty;

#[derive(Debug)]
pub struct TypeckOutput {
    expr_types: FxHashMap<SyntaxNodePtr, Ty>,
    field_method_access_info: FxHashMap<SyntaxNodePtr, FieldMethodAccessInfo>,
    contextual_types: FxHashMap<SyntaxNodePtr, Ty>,
    assoc_templates: Vec<AssocTemplateInfo>,
    errors: Vec<Error>, // TODO
}

#[derive(Debug)]
pub struct FieldMethodAccessInfo {
    pub base_ty: Ty,
    pub field_or_method: Option<FieldOrMethod>,
}

#[derive(Debug)]
pub struct AssocTemplateInfo {
    pub name: EcoString,
    pub defn_range: TextRange,
    pub context_ty: Ty,
}
