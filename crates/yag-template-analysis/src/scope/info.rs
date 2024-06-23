use std::ops::Index;

use ecow::EcoString;
use slotmap::{new_key_type, SlotMap};
use yag_template_syntax::ast::{self, AstNode};
use yag_template_syntax::{TextRange, TextSize};

new_key_type! { pub struct ScopeId; }

#[derive(Debug)]
pub struct ScopeInfo(SlotMap<ScopeId, Scope>);

impl ScopeInfo {
    pub(crate) fn new(scopes: SlotMap<ScopeId, Scope>) -> Self {
        Self(scopes)
    }
}

impl ScopeInfo {
    pub fn find_innermost_containing(&self, offset: TextSize) -> Option<ScopeId> {
        self.0
            .iter()
            .filter(|(_, scope)| scope.text_range.contains_inclusive(offset))
            .min_by_key(|(_, scope)| scope.text_range.len())
            .map(|(id, _)| id)
    }

    /// Iterate over the scopes containing the offset, from the innermost outward.
    pub fn scopes_containing(&self, offset: TextSize) -> ParentScopesIter {
        ParentScopesIter::new(self.find_innermost_containing(offset), self)
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

impl Iterator for ParentScopesIter<'_> {
    type Item = ScopeId;

    fn next(&mut self) -> Option<Self::Item> {
        let scope = self.cur.take()?;
        self.cur = self.info[scope].parent;
        Some(scope)
    }
}

impl Index<ScopeId> for ScopeInfo {
    type Output = Scope;

    fn index(&self, index: ScopeId) -> &Self::Output {
        &self.0[index]
    }
}

#[derive(Debug)]
pub struct Var {
    pub name: EcoString,
    pub visible_from: TextSize,
    pub decl_range: Option<TextRange>,
}

impl Var {
    pub(crate) fn new(name: impl Into<EcoString>, visible_from: TextSize, decl_range: Option<TextRange>) -> Self {
        Self {
            name: name.into(),
            visible_from,
            decl_range,
        }
    }

    pub(crate) fn try_from_decl(decl: ast::VarDecl) -> Option<Self> {
        let range = decl.syntax().text_range();
        let var = Self {
            name: decl.var()?.name().into(),
            visible_from: range.end(),
            decl_range: Some(range),
        };
        Some(var)
    }
}

#[derive(Debug)]
pub struct Scope {
    pub text_range: TextRange,
    pub vars: Vec<Var>, // sorted by source position
    pub parent: Option<ScopeId>,
}

impl Scope {
    pub fn new(text_range: TextRange, declared_vars: Vec<Var>, parent: Option<ScopeId>) -> Self {
        Self {
            text_range,
            vars: declared_vars,
            parent,
        }
    }

    pub fn vars_visible_at_offset(&self, offset: TextSize) -> impl Iterator<Item = &Var> {
        self.vars.iter().filter(move |var| offset >= var.visible_from)
    }
}
