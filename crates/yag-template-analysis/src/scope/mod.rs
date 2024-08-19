mod analysis;

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
    pub name: SmolStr,
    pub visible_from: TextSize,
    pub decl_range: Option<TextRange>,
}

#[derive(Debug)]
pub struct ScopeInfo {
    vars: SlotMap<DeclaredVarId, DeclaredVar>,
    resolved_var_uses: AHashMap<TextRange, DeclaredVarId>,
    scopes: SlotMap<ScopeId, Scope>,
}

impl ScopeInfo {
    pub(crate) fn new(
        vars: SlotMap<DeclaredVarId, DeclaredVar>,
        resolved_var_uses: AHashMap<TextRange, DeclaredVarId>,
        scopes: SlotMap<ScopeId, Scope>,
    ) -> Self {
        Self {
            vars,
            resolved_var_uses,
            scopes,
        }
    }
}

impl ScopeInfo {
    pub fn resolve_var_use(&self, var: ast::Var) -> Option<&DeclaredVar> {
        self.resolved_var_uses
            .get(&var.syntax().text_range())
            .and_then(|id| self.vars.get(*id))
    }

    /// Iterate over the scopes containing the offset, from the innermost outward.
    pub fn scopes_containing(&self, offset: TextSize) -> ParentScopesIter {
        ParentScopesIter::new(self.innermost_scope_containing(offset), self)
    }

    fn innermost_scope_containing(&self, offset: TextSize) -> Option<ScopeId> {
        self.scopes
            .iter()
            .filter(|(_, scope)| scope.range.contains_inclusive(offset))
            .min_by_key(|(_, scope)| scope.range.len())
            .map(|(id, _)| id)
    }
}

pub struct ParentScopesIter<'a> {
    info: &'a ScopeInfo,
    cur: Option<ScopeId>,
}

impl<'a> ParentScopesIter<'a> {
    pub(crate) fn new(innermost: Option<ScopeId>, info: &'a ScopeInfo) -> Self {
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
