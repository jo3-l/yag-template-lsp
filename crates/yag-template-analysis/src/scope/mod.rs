mod analysis;

use std::collections::hash_map;

use ahash::AHashMap;
pub use analysis::analyze;
use rowan::{TextRange, TextSize};
use slotmap::{new_key_type, SlotMap};
use smol_str::SmolStr;
use yag_template_syntax::ast;
use yag_template_syntax::ast::AstToken;

new_key_type! { pub struct ScopeId; }
new_key_type! { pub struct DeclaredVarId; }

#[derive(Debug)]
pub struct Scope {
    pub range: TextRange,
    pub vars_by_name: AHashMap<SmolStr, DeclaredVarId>,
    pub declared_vars: Vec<DeclaredVar>,
    pub parent: Option<ScopeId>,
}

impl Scope {
    pub fn new(range: TextRange, parent: Option<ScopeId>) -> Self {
        Self {
            range,
            vars_by_name: AHashMap::new(),
            declared_vars: Vec::new(),
            parent,
        }
    }

    pub fn lookup(&self, name: &str) -> Option<DeclaredVarId> {
        self.vars_by_name.get(name).copied()
    }

    pub fn vars_visible_at_offset(&self, offset: TextSize) -> impl Iterator<Item = &DeclaredVar> {
        self.declared_vars.iter().filter(move |var| offset >= var.visible_from)
    }
}

#[derive(Debug, Clone)]
pub struct DeclaredVar {
    pub id: DeclaredVarId,
    pub name: SmolStr,
    pub visible_from: TextSize, // within its scope, the variable is accessible after this offset
    pub decl_range: Option<TextRange>,
}

#[derive(Debug)]
pub struct ScopeInfo {
    declared_vars: SlotMap<DeclaredVarId, DeclaredVar>,
    resolved_var_references: AHashMap<TextRange, DeclaredVarId>,
    scopes: SlotMap<ScopeId, Scope>,
}

impl ScopeInfo {
    pub(crate) fn new(
        declared_vars: SlotMap<DeclaredVarId, DeclaredVar>,
        resolved_var_references: AHashMap<TextRange, DeclaredVarId>,
        scopes: SlotMap<ScopeId, Scope>,
    ) -> Self {
        Self {
            declared_vars,
            resolved_var_references,
            scopes,
        }
    }
}

impl ScopeInfo {
    pub fn resolve_var(&self, var: ast::Var) -> Option<&DeclaredVar> {
        self.resolved_var_references
            .get(&var.syntax().text_range())
            .and_then(|id| self.declared_vars.get(*id))
    }

    pub fn find_uses(&self, var: &DeclaredVar, include_decl: bool) -> VarUsesIter {
        VarUsesIter::new(self, var, include_decl)
    }

    /// Iterate over the scopes containing the offset, from the innermost outward.
    pub fn scopes_containing(&self, offset: TextSize) -> ParentScopesIter {
        ParentScopesIter::new(self, self.innermost_scope_containing(offset))
    }

    fn innermost_scope_containing(&self, offset: TextSize) -> Option<ScopeId> {
        self.scopes
            .iter()
            .filter(|(_, scope)| scope.range.contains_inclusive(offset))
            .min_by_key(|(_, scope)| scope.range.len())
            .map(|(id, _)| id)
    }
}

pub struct VarUsesIter<'a> {
    var: DeclaredVar,
    exclude_range: Option<TextRange>,
    inner: hash_map::Iter<'a, TextRange, DeclaredVarId>,
}

impl<'a> VarUsesIter<'a> {
    pub(crate) fn new(info: &'a ScopeInfo, var: &DeclaredVar, include_decl: bool) -> Self {
        Self {
            var: var.clone(),
            exclude_range: if include_decl { None } else { var.decl_range },
            inner: info.resolved_var_references.iter(),
        }
    }
}

impl<'a> Iterator for VarUsesIter<'a> {
    type Item = TextRange;

    fn next(&mut self) -> Option<Self::Item> {
        // This implementation is efficient than it could be; we iterate over all variable references whereas in theory
        // we could directly record the locations at which a variable is used in DeclaredVar. But that trades time for
        // memory and finding all references is an uncommon enough operation that the inefficiency here is OK.
        self.inner.find_map(|(&range, &id)| {
            if id == self.var.id
                && !self
                    .exclude_range
                    .is_some_and(|excluded| excluded.contains_range(range))
            {
                Some(range)
            } else {
                None
            }
        })
    }
}

pub struct ParentScopesIter<'a> {
    info: &'a ScopeInfo,
    cur: Option<ScopeId>,
}

impl<'a> ParentScopesIter<'a> {
    pub(crate) fn new(info: &'a ScopeInfo, innermost: Option<ScopeId>) -> Self {
        Self { info, cur: innermost }
    }
}

impl<'a> Iterator for ParentScopesIter<'a> {
    type Item = &'a Scope;

    fn next(&mut self) -> Option<Self::Item> {
        let scope_id = self.cur.take()?;
        let scope = &self.info.scopes[scope_id];
        self.cur = scope.parent;
        Some(scope)
    }
}
