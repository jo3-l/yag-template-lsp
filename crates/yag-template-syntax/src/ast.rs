pub use rowan::SyntaxText;

use crate::kind::SyntaxKind;
use crate::rowan_interface::{cast_children, cast_first_child};
pub use crate::rowan_interface::{AstChildren, AstNode, NodeOrToken, SyntaxElement, SyntaxNode, SyntaxToken};

macro_rules! define_node {
    ($(#[$attr:meta])* $name:ident($pat:pat)) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $(#[$attr])*
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            fn cast(node: SyntaxNode) -> Option<Self> {
                if matches!(node.kind(), $pat) {
                    Some(Self(node))
                } else {
                    None
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

define_node! {
    Root(SyntaxKind::Root)
}

impl Root {
    pub fn actions(&self) -> AstChildren<Action> {
        cast_children(self)
    }
}

define_node! {
    Text(SyntaxKind::Text)
}

impl Text {
    pub fn get(&self) -> SyntaxText {
        self.syntax().text()
    }
}

#[derive(Debug, Clone, Hash)]
pub enum Action {
    Text(Text),
    IfConditional(IfConditional),
    RangeLoop(RangeLoop),
    WhileLoop(WhileLoop),
    TryCatch(TryCatchAction),
    ExprAction(ExprAction),
}

impl AstNode for Action {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::Text => Text::cast(node).map(Self::Text),
            SyntaxKind::IfConditional => IfConditional::cast(node).map(Self::IfConditional),
            SyntaxKind::RangeLoop => RangeLoop::cast(node).map(Self::RangeLoop),
            SyntaxKind::WhileLoop => WhileLoop::cast(node).map(Self::WhileLoop),
            SyntaxKind::TryCatchAction => TryCatchAction::cast(node).map(Self::TryCatch),
            SyntaxKind::ExprAction => ExprAction::cast(node).map(Self::ExprAction),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Text(v) => v.syntax(),
            Self::IfConditional(v) => v.syntax(),
            Self::RangeLoop(v) => v.syntax(),
            Self::WhileLoop(v) => v.syntax(),
            Self::TryCatch(v) => v.syntax(),
            Self::ExprAction(v) => v.syntax(),
        }
    }
}

define_node! {
    ActionList(SyntaxKind::ActionList)
}

impl ActionList {
    pub fn actions(&self) -> AstChildren<Action> {
        cast_children(self)
    }
}

define_node! {
    LeftDelim(SyntaxKind::LeftDelim | SyntaxKind::TrimmedLeftDelim)
}

impl LeftDelim {
    pub fn has_trim_marker(&self) -> bool {
        self.syntax().kind() == SyntaxKind::TrimmedLeftDelim
    }
}

define_node! {
    RightDelim(SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim)
}

impl RightDelim {
    pub fn has_trim_marker(&self) -> bool {
        self.syntax().kind() == SyntaxKind::TrimmedRightDelim
    }
}

macro_rules! impl_delim_accessors {
    ($name:ident) => {
        impl $name {
            pub fn left_delim(&self) -> Option<LeftDelim> {
                cast_first_child(self)
            }

            pub fn right_delim(&self) -> Option<RightDelim> {
                cast_first_child(self)
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
        cast_first_child(self)
    }

    pub fn action_list(&self) -> Option<ActionList> {
        cast_first_child(self)
    }

    pub fn else_branches(&self) -> AstChildren<ElseBranch> {
        cast_children(self)
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        cast_first_child(self)
    }
}

define_node! {
    IfClause(SyntaxKind::IfClause)
}
impl_delim_accessors!(IfClause);

impl IfClause {
    pub fn if_expr(&self) -> Option<Expr> {
        cast_first_child(self)
    }
}

define_node! {
    ElseBranch(SyntaxKind::ElseBranch)
}

impl ElseBranch {
    pub fn else_clause(&self) -> Option<ElseClause> {
        cast_first_child(self)
    }

    pub fn action_list(&self) -> Option<ActionList> {
        cast_first_child(self)
    }
}

define_node! {
    ElseClause(SyntaxKind::ElseClause)
}
impl_delim_accessors!(ElseClause);

impl ElseClause {
    pub fn cond_expr(&self) -> Option<Expr> {
        cast_first_child(self)
    }
}

define_node! {
    RangeLoop(SyntaxKind::RangeLoop)
}

impl RangeLoop {
    pub fn range_clause(&self) -> Option<RangeClause> {
        cast_first_child(self)
    }

    pub fn action_list(&self) -> Option<ActionList> {
        cast_first_child(self)
    }

    pub fn else_branch(&self) -> Option<ElseBranch> {
        cast_first_child(self)
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        cast_first_child(self)
    }
}

define_node! {
    RangeClause(SyntaxKind::RangeClause)
}
impl_delim_accessors!(RangeClause);

impl RangeClause {
    pub fn iteration_vars(&self) -> AstChildren<Var> {
        cast_children(self)
    }

    pub fn range_expr(&self) -> Option<Expr> {
        cast_first_child(self)
    }

    pub fn eq_token(&self) -> Option<EqToken> {
        cast_first_child(self)
    }

    pub fn colon_eq_token(&self) -> Option<ColonEqToken> {
        cast_first_child(self)
    }
}

define_node! {
    EqToken(SyntaxKind::Eq)
}

define_node! {
    ColonEqToken(SyntaxKind::Eq)
}

define_node! {
    WhileLoop(SyntaxKind::WhileLoop)
}

impl WhileLoop {
    pub fn while_clause(&self) -> Option<WhileClause> {
        cast_first_child(self)
    }

    pub fn action_list(&self) -> Option<ActionList> {
        cast_first_child(self)
    }

    pub fn else_branch(&self) -> Option<ElseBranch> {
        cast_first_child(self)
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        cast_first_child(self)
    }
}

define_node! {
    WhileClause(SyntaxKind::WhileClause)
}
impl_delim_accessors!(WhileClause);

impl WhileClause {
    pub fn loop_condition_expr(&self) -> Option<Expr> {
        cast_first_child(self)
    }
}

define_node! {
    TryCatchAction(SyntaxKind::TryCatchAction)
}

impl TryCatchAction {
    pub fn try_clause(&self) -> Option<TryClause> {
        cast_first_child(self)
    }

    pub fn try_action_list(&self) -> Option<ActionList> {
        cast_first_child(self)
    }

    pub fn catch_clause(&self) -> Option<CatchClause> {
        cast_first_child(self)
    }

    pub fn catch_action_list(&self) -> Option<ActionList> {
        cast_children(self).nth(1)
    }
}

define_node! {
    TryClause(SyntaxKind::TryClause)
}
impl_delim_accessors!(TryClause);

define_node! {
    CatchClause(SyntaxKind::CatchClause)
}
impl_delim_accessors!(CatchClause);

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
            SyntaxKind::ContextFieldChain => ContextFieldChain::cast(node).map(Self::ContextFieldChain),
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
        self.syntax().text()
    }
}

define_node! {
    FuncCall(SyntaxKind::FuncCall)
}

impl FuncCall {
    pub fn func_name(&self) -> Option<Ident> {
        cast_first_child(self)
    }

    pub fn call_args(&self) -> AstChildren<Expr> {
        cast_children(self)
    }
}

define_node! {
    ExprCall(SyntaxKind::FuncCall)
}

impl ExprCall {
    pub fn callee(&self) -> Option<Expr> {
        cast_first_child(self)
    }

    pub fn call_args(&self) -> AstChildren<Expr> {
        cast_children(self)
    }
}

define_node! {
    ParenthesizedExpr(SyntaxKind::ParenthesizedExpr)
}

impl ParenthesizedExpr {
    pub fn inner_expr(&self) -> Option<Expr> {
        cast_first_child(self)
    }
}

define_node! {
    Pipeline(SyntaxKind::Pipeline)
}

impl Pipeline {
    pub fn init_expr(&self) -> Option<Expr> {
        cast_first_child(self)
    }

    pub fn stages(&self) -> AstChildren<PipelineStage> {
        cast_children(self)
    }
}

define_node! {
    PipelineStage(SyntaxKind::PipelineStage)
}

impl PipelineStage {
    pub fn target_expr(&self) -> Option<Expr> {
        cast_first_child(self)
    }
}

define_node! {
    ContextAccess(SyntaxKind::ContextAccess)
}

define_node! {
    Field(SyntaxKind::Field)
}

impl Field {
    pub fn name(&self) -> Option<SyntaxText> {
        let text = self.syntax().text();
        if text.char_at(0.into()) == Some('.') {
            Some(text.slice(1.into()..))
        } else {
            None
        }
    }
}

define_node! {
    ContextFieldChain(SyntaxKind::Field)
}

impl ContextFieldChain {
    pub fn fields(&self) -> AstChildren<Field> {
        cast_children(self)
    }
}

define_node! {
    ExprFieldChain(SyntaxKind::ExprFieldChain)
}

impl ExprFieldChain {
    pub fn fields(&self) -> AstChildren<Field> {
        cast_children(self)
    }
}

define_node! {
    Var(SyntaxKind::Var)
}

impl Var {
    pub fn name(&self) -> SyntaxText {
        self.syntax().text()
    }
}

define_node! {
    VarAccess(SyntaxKind::VarAccess)
}

impl VarAccess {
    pub fn var(&self) -> Option<Var> {
        cast_first_child(self)
    }
}

define_node! {
    VarDecl(SyntaxKind::VarDecl)
}

impl VarDecl {
    pub fn var(&self) -> Option<Var> {
        cast_first_child(self)
    }

    pub fn initializer(&self) -> Option<Expr> {
        cast_first_child(self)
    }
}

define_node! {
    VarAssign(SyntaxKind::VarAssign)
}

impl VarAssign {
    pub fn var(&self) -> Option<Var> {
        cast_first_child(self)
    }

    pub fn new_val(&self) -> Option<Expr> {
        cast_first_child(self)
    }
}

define_node! {
    Bool(SyntaxKind::Bool)
}

impl Bool {
    pub fn get(&self) -> bool {
        self.syntax().text() == "true"
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
