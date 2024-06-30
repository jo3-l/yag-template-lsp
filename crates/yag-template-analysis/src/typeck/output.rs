use ecow::EcoString;
use rustc_hash::FxHashMap;
use yag_template_syntax::{SyntaxNodePtr, TextRange};

use super::ty::foreign::FieldOrMethod;
use super::ty::Ty;
use super::Error;

#[derive(Debug, Default)]
pub struct TypeckOutput {
    pub(crate) expr_types: FxHashMap<SyntaxNodePtr, Ty>,
    pub(crate) field_method_access_info: FxHashMap<SyntaxNodePtr, FieldMethodAccessInfo>,
    pub(crate) contextual_types: FxHashMap<SyntaxNodePtr, Ty>,
    pub(crate) assoc_template_info: Vec<AssocTemplateInfo>,
    pub(crate) errors: Vec<Error>, // TODO
}

#[derive(Debug)]
pub struct FieldMethodAccessInfo {
    pub base_ty: Ty,
    pub resolved: Option<FieldOrMethod>,
}

#[derive(Debug)]
pub struct AssocTemplateInfo {
    pub name: EcoString,
    pub defn_range: TextRange,
    pub context_ty: Ty,
    pub return_ty: Ty,
}
