use rowan::TextSize;

use crate::ast::{AstNode, AstToken, SyntaxNodeExt};
use crate::{ast, SyntaxKind, SyntaxNode, SyntaxToken};

pub struct Query {
    pub offset: TextSize,
    /// The token before the cursor.
    ///
    /// At edges, we do not know whether the token of interest lies to the left or to the right of the cursor, so we
    /// need to check both. Consider:
    ///
    /// ```txt
    /// case 1:
    /// .   |func
    ///      ^^^^ interesting token to the right
    ///
    /// case 2:
    /// .   func|
    ///     ^^^^ interesting token to the left
    /// ```
    pub before: Option<SyntaxToken>,
    /// The token after the cursor.
    pub after: Option<SyntaxToken>,
}

impl Query {
    pub fn at(root: &SyntaxNode, offset: TextSize) -> Self {
        let t = root.token_at_offset(offset);
        Self {
            offset,
            before: t.clone().left_biased(),
            after: t.right_biased(),
        }
    }

    pub fn matches(&self, f: impl Fn(&SyntaxToken) -> bool) -> bool {
        self.before.as_ref().is_some_and(&f) || self.after.as_ref().is_some_and(f)
    }

    pub fn map<F, R>(&self, f: F) -> Option<R>
    where
        F: Fn(&SyntaxToken) -> Option<R>,
    {
        match self.before.as_ref().and_then(&f) {
            Some(mapped) => Some(mapped),
            None => self.after.as_ref().and_then(f),
        }
    }
}

impl Query {
    pub fn is_in_var_access(&self) -> bool {
        self.matches(|tok| {
            tok.kind() == SyntaxKind::Var && tok.parent().is_some_and(|parent| parent.is::<ast::VarAccess>())
        })
    }

    pub fn var(&self) -> Option<ast::Var> {
        self.map(|tok| ast::Var::cast(tok.clone()))
    }

    pub fn is_in_func_call(&self) -> bool {
        self.matches(|tok| {
            tok.kind() == SyntaxKind::Ident && tok.parent().is_some_and(|parent| parent.is::<ast::FuncCall>())
        })
    }

    pub fn ident(&self) -> Option<ast::Ident> {
        self.map(|tok| ast::Ident::cast(tok.clone()))
    }

    pub fn parent_expr(&self) -> Option<ast::Expr> {
        self.map(|tok| tok.parent_ancestors().find_map(ast::Expr::cast))
    }
}
