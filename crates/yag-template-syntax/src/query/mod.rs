use crate::ast::{self, AstNode, AstToken, SyntaxNodeExt};
use crate::{SyntaxKind, SyntaxNode, SyntaxToken, TextSize};

pub struct Query {
    pub offset: TextSize,
    /// The token right before the cursor.
    pub before: SyntaxToken,
}

impl Query {
    pub fn at(root: &SyntaxNode, offset: TextSize) -> Option<Self> {
        let before = root.token_at_offset(offset).left_biased()?;
        let query = Query { offset, before };
        Some(query)
    }
}

impl Query {
    pub fn is_var_access(&self) -> bool {
        self.before.kind() == SyntaxKind::Var
            && self.before.parent().is_some_and(|parent| parent.is::<ast::VarAccess>())
    }

    pub fn var(&self) -> Option<ast::Var> {
        ast::Var::cast(self.before.clone())
    }

    pub fn can_complete_fn_name(&self) -> bool {
        self.before.kind() == SyntaxKind::Ident
            && self.before.parent().is_some_and(|parent| parent.is::<ast::FuncCall>())
    }

    pub fn ident(&self) -> Option<ast::Ident> {
        ast::Ident::cast(self.before.clone())
    }

    pub fn parent_expr(&self) -> Option<ast::Expr> {
        self.before.parent_ancestors().find_map(ast::Expr::cast)
    }
}
