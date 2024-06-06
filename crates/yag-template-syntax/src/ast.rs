use crate::ast_support::{cast_children, cast_first_child, define_node};
use crate::{AstChildren, AstNode, SyntaxKind, SyntaxNode, SyntaxText};

define_node! {
    Root(SyntaxKind::Root)
}

impl Root {
    pub fn actions(&self) -> AstChildren<Action> {
        cast_children(self.syntax())
    }
}

define_node! {
    Text(SyntaxKind::Text)
}

impl Text {
    pub fn get(&self) -> SyntaxText {
        self.0.text()
    }
}

#[derive(Debug, Clone, Hash)]
pub enum Action {
    Text(Text),
    IfConditional(IfConditional),
    ExprAction(ExprAction),
}

impl AstNode for Action {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::Text => Text::cast(node).map(Self::Text),
            SyntaxKind::IfConditional => IfConditional::cast(node).map(Self::IfConditional),
            SyntaxKind::ExprAction => ExprAction::cast(node).map(Self::ExprAction),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Text(v) => v.syntax(),
            Self::IfConditional(v) => v.syntax(),
            Self::ExprAction(v) => v.syntax(),
        }
    }
}

define_node! {
    ActionList(SyntaxKind::ActionList)
}

impl ActionList {
    pub fn actions(&self) -> AstChildren<Action> {
        cast_children(self.syntax())
    }
}

define_node! {
    LeftDelim(SyntaxKind::LeftDelim | SyntaxKind::TrimmedLeftDelim)
}

impl LeftDelim {
    pub fn has_trim_marker(&self) -> bool {
        self.0.kind() == SyntaxKind::TrimmedLeftDelim
    }
}

define_node! {
    RightDelim(SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim)
}

impl RightDelim {
    pub fn has_trim_marker(&self) -> bool {
        self.0.kind() == SyntaxKind::TrimmedRightDelim
    }
}

macro_rules! impl_delimiter_accessors {
    ($name:ident) => {
        impl $name {
            pub fn left_delim(&self) -> Option<LeftDelim> {
                cast_first_child(self.syntax())
            }

            pub fn right_delim(&self) -> Option<RightDelim> {
                cast_first_child(self.syntax())
            }
        }
    };
}

define_node! {
    EndClause(SyntaxKind::EndClause)
}
impl_delimiter_accessors!(EndClause);

define_node! {
    IfConditional(SyntaxKind::IfConditional)
}

impl IfConditional {
    pub fn if_clause(&self) -> Option<IfClause> {
        cast_first_child(self.syntax())
    }

    pub fn action_list(&self) -> Option<ActionList> {
        cast_first_child(self.syntax())
    }

    pub fn else_branches(&self) -> AstChildren<ElseBranch> {
        cast_children(self.syntax())
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    IfClause(SyntaxKind::IfClause)
}
impl_delimiter_accessors!(IfClause);

define_node! {
    ElseBranch(SyntaxKind::ElseBranch)
}

impl ElseBranch {
    pub fn clause(&self) -> Option<ElseClause> {
        cast_first_child(self.syntax())
    }

    pub fn action_list(&self) -> Option<ActionList> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    ElseClause(SyntaxKind::ElseClause)
}
impl_delimiter_accessors!(ElseClause);

impl ElseClause {
    pub fn cond(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    ExprAction(SyntaxKind::ExprAction)
}
impl_delimiter_accessors!(ExprAction);

#[derive(Debug, Clone, Hash)]
pub enum Expr {
    FuncCall(FuncCall),
    VarRef(VarRef),
    VarDecl(VarDecl),
    VarAssign(VarAssign),
    Bool(Bool),
    Int(Int),
}

impl AstNode for Expr {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::FuncCall => FuncCall::cast(node).map(Self::FuncCall),
            SyntaxKind::VarRef => VarRef::cast(node).map(Self::VarRef),
            SyntaxKind::VarDecl => VarDecl::cast(node).map(Self::VarDecl),
            SyntaxKind::VarAssign => VarAssign::cast(node).map(Self::VarAssign),
            SyntaxKind::Bool => Bool::cast(node).map(Self::Bool),
            SyntaxKind::Int => Int::cast(node).map(Self::Int),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Expr::FuncCall(v) => v.syntax(),
            Expr::VarRef(v) => v.syntax(),
            Expr::VarDecl(v) => v.syntax(),
            Expr::VarAssign(v) => v.syntax(),
            Expr::Bool(v) => v.syntax(),
            Expr::Int(v) => v.syntax(),
        }
    }
}

define_node! {
    Ident(SyntaxKind::Ident)
}

impl Ident {
    pub fn get(&self) -> SyntaxText {
        self.0.text()
    }
}

define_node! {
    FuncCall(SyntaxKind::FuncCall)
}

impl FuncCall {
    pub fn fn_ident(&self) -> Option<Ident> {
        cast_first_child(self.syntax())
    }

    pub fn call_args(&self) -> AstChildren<Expr> {
        cast_children(self.syntax())
    }
}

define_node! {
    Var(SyntaxKind::Var)
}

impl Var {
    pub fn name(&self) -> SyntaxText {
        self.0.text()
    }
}

define_node! {
    VarRef(SyntaxKind::VarRef)
}

impl VarRef {
    pub fn var(&self) -> Option<Var> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    VarDecl(SyntaxKind::VarDecl)
}

impl VarDecl {
    pub fn var(&self) -> Option<Var> {
        cast_first_child(self.syntax())
    }

    pub fn initializer(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    VarAssign(SyntaxKind::VarAssign)
}

impl VarAssign {
    pub fn var(&self) -> Option<Var> {
        cast_first_child(self.syntax())
    }

    pub fn new_val(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    Bool(SyntaxKind::Bool)
}

impl Bool {
    pub fn get(&self) -> bool {
        self.0.text() == "true"
    }
}

define_node! {
    Int(SyntaxKind::Int)
}

impl Int {
    pub fn get(&self) -> i64 {
        todo!()
    }
}
