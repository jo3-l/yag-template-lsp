mod analysis;

use std::collections::hash_map;

pub use analysis::analyze;
use foldhash::{HashMap, HashMapExt};
use rowan::{TextRange, TextSize};
use slotmap::{new_key_type, SlotMap};
use smol_str::SmolStr;
use yag_template_syntax::ast;
use yag_template_syntax::ast::AstToken;

new_key_type! { pub struct VarSymbolId; }
new_key_type! { pub struct ScopeId; }

#[derive(Debug)]
pub struct ScopeInfo {
    var_syms: SlotMap<VarSymbolId, VarSymbol>,
    resolved_var_uses: HashMap<TextRange, VarSymbolId>, // indexed by text range of ast::Var
    scopes: SlotMap<ScopeId, Scope>,
}

impl ScopeInfo {
    pub(crate) fn new(
        var_syms: SlotMap<VarSymbolId, VarSymbol>,
        resolved_var_uses: HashMap<TextRange, VarSymbolId>,
        scopes: SlotMap<ScopeId, Scope>,
    ) -> Self {
        Self {
            var_syms,
            resolved_var_uses,
            scopes,
        }
    }

    pub fn resolve_var(&self, var: ast::Var) -> Option<&VarSymbol> {
        self.resolved_var_uses
            .get(&var.syntax().text_range())
            .and_then(|id| self.var_syms.get(*id))
    }

    pub fn find_uses(&self, sym: &VarSymbol, include_decl: bool) -> VarUsesIter {
        VarUsesIter::new(self, sym, include_decl)
    }

    /// Iterate over the scopes containing the offset, from the innermost outward.
    pub fn scopes_containing(&self, offset: TextSize) -> ParentScopesIter {
        ParentScopesIter::new(self, self.innermost_scope_containing(offset))
    }

    pub fn innermost_scope_containing(&self, offset: TextSize) -> Option<ScopeId> {
        self.scopes
            .iter()
            .filter(|(_, scope)| scope.range.contains_inclusive(offset))
            .min_by_key(|(_, scope)| scope.range.len())
            .map(|(id, _)| id)
    }
}

#[derive(Debug, Clone)]
pub struct VarSymbol {
    pub id: VarSymbolId,
    pub name: SmolStr,
    /// Within its scope, the variable is accessible after this offset.
    pub visible_from: TextSize,
    pub decl_range: Option<TextRange>,
}

#[derive(Debug)]
pub struct Scope {
    pub range: TextRange,
    pub vars_by_name: HashMap<SmolStr, VarSymbolId>,
    pub declared_vars: Vec<VarSymbol>,
    pub parent: Option<ScopeId>,
}

impl Scope {
    pub(crate) fn new(range: TextRange, parent: Option<ScopeId>) -> Self {
        Self {
            range,
            vars_by_name: HashMap::new(),
            declared_vars: Vec::new(),
            parent,
        }
    }

    pub fn lookup(&self, name: &str) -> Option<VarSymbolId> {
        self.vars_by_name.get(name).copied()
    }

    pub fn vars_visible_at_offset(&self, offset: TextSize) -> impl Iterator<Item = &VarSymbol> {
        self.declared_vars.iter().filter(move |var| offset >= var.visible_from)
    }
}

pub struct VarUsesIter<'a> {
    sym: VarSymbol,
    exclude_range: Option<TextRange>,
    all_var_uses: hash_map::Iter<'a, TextRange, VarSymbolId>,
}

impl<'a> VarUsesIter<'a> {
    pub(crate) fn new(info: &'a ScopeInfo, sym: &VarSymbol, include_decl: bool) -> Self {
        Self {
            sym: sym.clone(),
            exclude_range: if include_decl { None } else { sym.decl_range },
            all_var_uses: info.resolved_var_uses.iter(),
        }
    }
}

impl<'a> Iterator for VarUsesIter<'a> {
    type Item = TextRange;

    fn next(&mut self) -> Option<Self::Item> {
        // This implementation is less efficient than it could be; we iterate over all variable references whereas in
        // theory we could directly record the locations at which a variable is used in DeclaredVar. But that trades
        // time for memory and makes cloning a DeclaredVar somewhat expensive, so on balance the current strategy seems
        // acceptable too.
        self.all_var_uses.find_map(|(&range, &id)| {
            if id == self.sym.id
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
        let scope = &self.info.scopes[self.cur.take()?];
        self.cur = scope.parent;
        Some(scope)
    }
}
