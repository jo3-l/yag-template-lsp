use crate::ast::tokens::*;
use crate::ast::{cast_children, cast_first_child, AstElement, AstElementChildren};
use crate::{SyntaxElement, SyntaxKind, SyntaxNode};

macro_rules! define_node {
    ($(#[$attr:meta])* $name:ident($pat:pat)) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $(#[$attr])*
        pub struct $name {
            syntax: SyntaxNode,
        }

        impl AstElement for $name {
            fn cast(element: SyntaxElement) -> Option<Self> {
                element.into_node().and_then(|node| {
                    if matches!(node.kind(), $pat) {
                        Some(Self { syntax: node })
                    } else {
                        None
                    }
                })
            }
        }

        impl $name {
            pub fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
        }
    };
}

define_node! {
    Root(SyntaxKind::Root)
}

impl Root {
    pub fn actions(&self) -> AstElementChildren<Action> {
        cast_children(self.syntax())
    }
}

#[derive(Debug, Clone, Hash)]
pub enum Action {
    Text(Text),
    TemplateDefinition(TemplateDefinition),
    TemplateBlock(TemplateBlock),
    TemplateInvocation(TemplateInvocation),
    Return(ReturnAction),
    IfConditional(IfConditional),
    WithConditional(WithConditional),
    RangeLoop(RangeLoop),
    WhileLoop(WhileLoop),
    LoopBreak(LoopBreak),
    LoopContinue(LoopContinue),
    TryCatch(TryCatchAction),
    ExprAction(ExprAction),
}

impl AstElement for Action {
    fn cast(element: SyntaxElement) -> Option<Self> {
        match element.kind() {
            SyntaxKind::Text => Text::cast(element).map(Self::Text),
            SyntaxKind::TemplateDefinition => TemplateDefinition::cast(element).map(Self::TemplateDefinition),
            SyntaxKind::TemplateBlock => TemplateBlock::cast(element).map(Self::TemplateBlock),
            SyntaxKind::TemplateInvocation => TemplateInvocation::cast(element).map(Self::TemplateInvocation),
            SyntaxKind::Return => ReturnAction::cast(element).map(Self::Return),
            SyntaxKind::IfConditional => IfConditional::cast(element).map(Self::IfConditional),
            SyntaxKind::WithConditional => WithConditional::cast(element).map(Self::WithConditional),
            SyntaxKind::RangeLoop => RangeLoop::cast(element).map(Self::RangeLoop),
            SyntaxKind::WhileLoop => WhileLoop::cast(element).map(Self::WhileLoop),
            SyntaxKind::LoopBreak => LoopBreak::cast(element).map(Self::LoopBreak),
            SyntaxKind::LoopContinue => LoopContinue::cast(element).map(Self::LoopContinue),
            SyntaxKind::TryCatchAction => TryCatchAction::cast(element).map(Self::TryCatch),
            SyntaxKind::ExprAction => ExprAction::cast(element).map(Self::ExprAction),
            _ => None,
        }
    }
}

impl Action {
    pub fn syntax(&self) -> SyntaxElement {
        match self {
            Self::Text(v) => v.syntax().clone().into(),
            Self::TemplateDefinition(v) => v.syntax().clone().into(),
            Self::TemplateBlock(v) => v.syntax().clone().into(),
            Self::TemplateInvocation(v) => v.syntax().clone().into(),
            Self::Return(v) => v.syntax().clone().into(),
            Self::IfConditional(v) => v.syntax().clone().into(),
            Self::WithConditional(v) => v.syntax().clone().into(),
            Self::RangeLoop(v) => v.syntax().clone().into(),
            Self::WhileLoop(v) => v.syntax().clone().into(),
            Self::LoopBreak(v) => v.syntax().clone().into(),
            Self::LoopContinue(v) => v.syntax().clone().into(),
            Self::TryCatch(v) => v.syntax().clone().into(),
            Self::ExprAction(v) => v.syntax().clone().into(),
        }
    }
}

macro_rules! delim_accessors {
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
    ActionList(SyntaxKind::ActionList)
}

impl ActionList {
    pub fn actions(&self) -> AstElementChildren<Action> {
        cast_children(self.syntax())
    }
}

define_node! {
    TemplateDefinition(SyntaxKind::TemplateDefinition)
}

impl TemplateDefinition {
    pub fn define_clause(&self) -> Option<DefineClause> {
        cast_first_child(self.syntax())
    }

    pub fn action_list(&self) -> Option<ActionList> {
        cast_first_child(self.syntax())
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    DefineClause(SyntaxKind::DefineClause)
}
delim_accessors!(DefineClause);

impl DefineClause {
    pub fn template_name(&self) -> Option<StringLiteral> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    TemplateBlock(SyntaxKind::TemplateBlock)
}

impl TemplateBlock {
    pub fn block_clause(&self) -> Option<BlockClause> {
        cast_first_child(self.syntax())
    }

    pub fn action_list(&self) -> Option<ActionList> {
        cast_first_child(self.syntax())
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    BlockClause(SyntaxKind::BlockClause)
}
delim_accessors!(BlockClause);

impl BlockClause {
    pub fn template_name(&self) -> Option<StringLiteral> {
        cast_first_child(self.syntax())
    }

    pub fn context_data_expr(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    TemplateInvocation(SyntaxKind::TemplateInvocation)
}
delim_accessors!(TemplateInvocation);

impl TemplateInvocation {
    pub fn template_name(&self) -> Option<StringLiteral> {
        cast_first_child(self.syntax())
    }

    pub fn context_data_expr(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    ReturnAction(SyntaxKind::ReturnAction)
}
delim_accessors!(ReturnAction);

impl ReturnAction {
    pub fn return_expr(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    EndClause(SyntaxKind::EndClause)
}
delim_accessors!(EndClause);

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

    pub fn else_branches(&self) -> AstElementChildren<ElseBranch> {
        cast_children(self.syntax())
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    IfClause(SyntaxKind::IfClause)
}
delim_accessors!(IfClause);

impl IfClause {
    pub fn if_expr(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

// XXX: Consider unifying IfConditional/WithConditional?
define_node! {
    WithConditional(SyntaxKind::WithConditional)
}

impl WithConditional {
    pub fn with_clause(&self) -> Option<WithClause> {
        cast_first_child(self.syntax())
    }

    pub fn action_list(&self) -> Option<ActionList> {
        cast_first_child(self.syntax())
    }

    pub fn else_branches(&self) -> AstElementChildren<ElseBranch> {
        cast_children(self.syntax())
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    WithClause(SyntaxKind::WithClause)
}
delim_accessors!(WithClause);

impl WithClause {
    pub fn with_expr(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    ElseBranch(SyntaxKind::ElseBranch)
}

impl ElseBranch {
    pub fn else_clause(&self) -> Option<ElseClause> {
        cast_first_child(self.syntax())
    }

    pub fn action_list(&self) -> Option<ActionList> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    ElseClause(SyntaxKind::ElseClause)
}
delim_accessors!(ElseClause);

impl ElseClause {
    pub fn cond_expr(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    RangeLoop(SyntaxKind::RangeLoop)
}

impl RangeLoop {
    pub fn range_clause(&self) -> Option<RangeClause> {
        cast_first_child(self.syntax())
    }

    pub fn action_list(&self) -> Option<ActionList> {
        cast_first_child(self.syntax())
    }

    pub fn else_branch(&self) -> Option<ElseBranch> {
        cast_first_child(self.syntax())
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    RangeClause(SyntaxKind::RangeClause)
}
delim_accessors!(RangeClause);

impl RangeClause {
    pub fn iteration_vars(&self) -> AstElementChildren<Var> {
        cast_children(self.syntax())
    }

    pub fn range_expr(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }

    pub fn declares_vars(&self) -> bool {
        self.syntax
            .children_with_tokens()
            .any(|el| el.into_token().is_some_and(|token| token.kind() == SyntaxKind::ColonEq))
    }

    pub fn assigns_vars(&self) -> bool {
        self.syntax
            .children_with_tokens()
            .any(|el| el.into_token().is_some_and(|token| token.kind() == SyntaxKind::Eq))
    }
}

define_node! {
    WhileLoop(SyntaxKind::WhileLoop)
}

impl WhileLoop {
    pub fn while_clause(&self) -> Option<WhileClause> {
        cast_first_child(self.syntax())
    }

    pub fn action_list(&self) -> Option<ActionList> {
        cast_first_child(self.syntax())
    }

    pub fn else_branch(&self) -> Option<ElseBranch> {
        cast_first_child(self.syntax())
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    WhileClause(SyntaxKind::WhileClause)
}
delim_accessors!(WhileClause);

impl WhileClause {
    pub fn loop_condition_expr(&self) -> Option<Expr> {
        cast_first_child(self.syntax())
    }
}

define_node! {
    LoopBreak(SyntaxKind::LoopBreak)
}
delim_accessors!(LoopBreak);

define_node! {
    LoopContinue(SyntaxKind::LoopContinue)
}
delim_accessors!(LoopContinue);

define_node! {
    TryCatchAction(SyntaxKind::TryCatchAction)
}

impl TryCatchAction {
    pub fn try_clause(&self) -> Option<TryClause> {
        cast_first_child(self.syntax())
    }

    pub fn try_action_list(&self) -> Option<ActionList> {
        cast_first_child(self.syntax())
    }

    pub fn catch_clause(&self) -> Option<CatchClause> {
        cast_first_child(self.syntax())
    }

    pub fn catch_action_list(&self) -> Option<ActionList> {
        cast_children(self.syntax()).nth(1)
    }
}

define_node! {
    TryClause(SyntaxKind::TryClause)
}
delim_accessors!(TryClause);

define_node! {
    CatchClause(SyntaxKind::CatchClause)
}
delim_accessors!(CatchClause);

define_node! {
    ExprAction(SyntaxKind::ExprAction)
}
delim_accessors!(ExprAction);

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
    Float(Float),
    Char(Char),
    StringLiteral(StringLiteral),
    Nil(Nil),
}

impl AstElement for Expr {
    fn cast(element: SyntaxElement) -> Option<Self> {
        match element.kind() {
            SyntaxKind::FuncCall => FuncCall::cast(element).map(Self::FuncCall),
            SyntaxKind::ExprCall => ExprCall::cast(element).map(Self::ExprCall),
            SyntaxKind::ParenthesizedExpr => ParenthesizedExpr::cast(element).map(Self::Parenthesized),
            SyntaxKind::Pipeline => Pipeline::cast(element).map(Self::Pipeline),
            SyntaxKind::ContextAccess => ContextAccess::cast(element).map(Self::ContextAccess),
            SyntaxKind::ContextFieldChain => ContextFieldChain::cast(element).map(Self::ContextFieldChain),
            SyntaxKind::ExprFieldChain => ExprFieldChain::cast(element).map(Self::ExprFieldChain),
            SyntaxKind::VarAccess => VarAccess::cast(element).map(Self::VarAccess),
            SyntaxKind::VarDecl => VarDecl::cast(element).map(Self::VarDecl),
            SyntaxKind::VarAssign => VarAssign::cast(element).map(Self::VarAssign),
            SyntaxKind::Bool => Bool::cast(element).map(Self::Bool),
            SyntaxKind::Int => Int::cast(element).map(Self::Int),
            SyntaxKind::Float => Float::cast(element).map(Self::Float),
            SyntaxKind::Char => Char::cast(element).map(Self::Char),
            SyntaxKind::InterpretedString | SyntaxKind::RawString => {
                StringLiteral::cast(element).map(Self::StringLiteral)
            }
            SyntaxKind::Nil => Nil::cast(element).map(Self::Nil),
            _ => None,
        }
    }
}

impl Expr {
    pub fn syntax(&self) -> SyntaxElement {
        match self {
            Self::FuncCall(v) => v.syntax().clone().into(),
            Self::ExprCall(v) => v.syntax().clone().into(),
            Self::Parenthesized(v) => v.syntax().clone().into(),
            Self::Pipeline(v) => v.syntax().clone().into(),
            Self::ContextAccess(v) => v.syntax().clone().into(),
            Self::ContextFieldChain(v) => v.syntax().clone().into(),
            Self::ExprFieldChain(v) => v.syntax().clone().into(),
            Self::VarAccess(v) => v.syntax().clone().into(),
            Self::VarDecl(v) => v.syntax().clone().into(),
            Self::VarAssign(v) => v.syntax().clone().into(),
            Self::Bool(v) => v.syntax().clone().into(),
            Self::Int(v) => v.syntax().clone().into(),
            Self::Float(v) => v.syntax().clone().into(),
            Self::Char(v) => v.syntax().clone().into(),
            Self::StringLiteral(v) => v.syntax().clone().into(),
            Self::Nil(v) => v.syntax().clone().into(),
        }
    }
}

define_node! {
    FuncCall(SyntaxKind::FuncCall)
}

impl FuncCall {
    pub fn func_name(&self) -> Option<Ident> {
        cast_first_child(self.syntax())
    }

    pub fn call_args(&self) -> AstElementChildren<Expr> {
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

    pub fn call_args(&self) -> AstElementChildren<Expr> {
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

    pub fn stages(&self) -> AstElementChildren<PipelineStage> {
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
    ContextFieldChain(SyntaxKind::ContextFieldChain)
}

impl ContextFieldChain {
    pub fn fields(&self) -> AstElementChildren<Field> {
        cast_children(self.syntax())
    }
}

define_node! {
    ExprFieldChain(SyntaxKind::ExprFieldChain)
}

impl ExprFieldChain {
    pub fn fields(&self) -> AstElementChildren<Field> {
        cast_children(self.syntax())
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
