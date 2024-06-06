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

macro_rules! impl_delim_accessors {
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
impl_delim_accessors!(EndClause);

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
impl_delim_accessors!(IfClause);

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
impl_delim_accessors!(ElseClause);

impl ElseClause {
    pub fn cond(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    ExprAction(SyntaxKind::ExprAction)
}
impl_delim_accessors!(ExprAction);

#[derive(Debug, Clone, Hash)]
pub enum Expr {
    FuncCall(FuncCall),
    ExprCall(ExprCall),
    Parenthesized(ParenthesizedExpr),
    Pipeline(Pipeline),
    ContextAccess(ContextAccess),
    ContextFieldChain(ContextFieldChain),
    ExprFieldChain(ExprFieldChain),
    VarAccess(VarAccess),
    VarDecl(VarDecl),
    VarAssign(VarAssign),
    Bool(Bool),
    Int(Int),
}

impl AstNode for Expr {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::FuncCall => FuncCall::cast(node).map(Self::FuncCall),
            SyntaxKind::ExprCall => ExprCall::cast(node).map(Self::ExprCall),
            SyntaxKind::ParenthesizedExpr => ParenthesizedExpr::cast(node).map(Self::Parenthesized),
            SyntaxKind::Pipeline => Pipeline::cast(node).map(Self::Pipeline),
            SyntaxKind::ContextAccess => ContextAccess::cast(node).map(Self::ContextAccess),
            SyntaxKind::ContextFieldChain => {
                ContextFieldChain::cast(node).map(Self::ContextFieldChain)
            }
            SyntaxKind::ExprFieldChain => ExprFieldChain::cast(node).map(Self::ExprFieldChain),
            SyntaxKind::VarAccess => VarAccess::cast(node).map(Self::VarAccess),
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
            Expr::ExprCall(v) => v.syntax(),
            Expr::Parenthesized(v) => v.syntax(),
            Expr::Pipeline(v) => v.syntax(),
            Expr::ContextAccess(v) => v.syntax(),
            Expr::ContextFieldChain(v) => v.syntax(),
            Expr::ExprFieldChain(v) => v.syntax(),
            Expr::VarAccess(v) => v.syntax(),
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
    pub fn func_name(&self) -> Option<Ident> {
        cast_first_child(self.syntax())
    }

    pub fn call_args(&self) -> AstChildren<Expr> {
        cast_children(self.syntax())
    }
}

define_node! {
    ExprCall(SyntaxKind::FuncCall)
}

impl ExprCall {
    pub fn callee(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }

    pub fn call_args(&self) -> AstChildren<Expr> {
        cast_children(self.syntax())
    }
}

define_node! {
    ParenthesizedExpr(SyntaxKind::ParenthesizedExpr)
}

impl ParenthesizedExpr {
    pub fn inner_expr(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    Pipeline(SyntaxKind::Pipeline)
}

impl Pipeline {
    pub fn init_expr(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }

    pub fn stages(&self) -> AstChildren<PipelineStage> {
        cast_children(self.syntax())
    }
}

define_node! {
    PipelineStage(SyntaxKind::PipelineStage)
}

impl PipelineStage {
    pub fn target_expr(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    ContextAccess(SyntaxKind::ContextAccess)
}

define_node! {
    Field(SyntaxKind::Field)
}

impl Field {
    pub fn ident(&self) -> Option<Field> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    ContextFieldChain(SyntaxKind::Field)
}

impl ContextFieldChain {
    pub fn fields(&self) -> AstChildren<Field> {
        cast_children(self.syntax())
    }
}

define_node! {
    ExprFieldChain(SyntaxKind::ExprFieldChain)
}

impl ExprFieldChain {
    pub fn fields(&self) -> AstChildren<Field> {
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
    VarAccess(SyntaxKind::VarAccess)
}

impl VarAccess {
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
