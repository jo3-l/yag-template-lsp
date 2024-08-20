use rowan::TextSize;

use crate::ast::{AstNode, AstToken, SyntaxNodeExt};
use crate::{ast, SyntaxKind, SyntaxNode, SyntaxToken};

pub struct Query {
    pub offset: TextSize,
    /// The token at the offset (left-biased in the case where the offset sits
    /// between two tokens.)
    pub token: SyntaxToken,
}

impl Query {
    pub fn at(root: &SyntaxNode, offset: TextSize) -> Option<Self> {
        let before = root.token_at_offset(offset).left_biased()?;
        let query = Query { offset, token: before };
        Some(query)
    }
}

impl Query {
    pub fn is_var_access(&self) -> bool {
        self.token.kind() == SyntaxKind::Var && self.token.parent().is_some_and(|parent| parent.is::<ast::VarAccess>())
    }

    pub fn var(&self) -> Option<ast::Var> {
        ast::Var::cast(self.token.clone())
    }

    pub fn is_func_call(&self) -> bool {
        self.token.kind() == SyntaxKind::Ident && self.token.parent().is_some_and(|parent| parent.is::<ast::FuncCall>())
    }

    pub fn ident(&self) -> Option<ast::Ident> {
        ast::Ident::cast(self.token.clone())
    }

    pub fn parent_expr(&self) -> Option<ast::Expr> {
        self.token.parent_ancestors().find_map(ast::Expr::cast)
    }
}
