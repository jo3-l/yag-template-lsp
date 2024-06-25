use ecow::EcoString;
use rustc_hash::{FxHashMap, FxHashSet};

use super::ty::Ty;

#[derive(Debug)]
pub(crate) struct VarAssignInfo {
    /// The union of all potential types for this variable, assuming that a path passing through
    /// an assignment was taken.
    pub(crate) ty: Ty,
    /// Whether an assignment to this variable occurs in all paths through this block.
    pub(crate) occurs_in_all_paths: bool,
}

#[derive(Debug)]
pub(crate) struct Block {
    /// The context type at the beginning of this block.
    pub(crate) initial_context_ty: Ty,
    /// The variable names declared within this block.
    pub(crate) declared_vars: FxHashSet<EcoString>,
    /// The potential variable assignments occurring in this block.
    pub(crate) var_assigns: FxHashMap<EcoString, VarAssignInfo>,
    /// Types of variables declared or potentially assigned to within this block. When looking up
    /// variable types, it is still necessary to examine parent blocks as there may be inherited
    /// variables.
    pub(crate) resolved_var_types: FxHashMap<EcoString, Ty>,
}

impl Block {
    pub(crate) fn new(context_ty: Ty) -> Self {
        Self {
            initial_context_ty: context_ty,
            declared_vars: FxHashSet::default(),
            var_assigns: FxHashMap::default(),
            resolved_var_types: FxHashMap::default(),
        }
    }
}
