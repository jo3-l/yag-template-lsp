use std::iter::Skip;

use rowan::SyntaxElementChildren;

use super::AstTokenChildren;
use crate::ast::{tokens, AstChildren, AstNode, AstToken, SyntaxNodeExt};
use crate::{SyntaxElement, SyntaxKind, SyntaxNode, YagTemplateLanguage};

macro_rules! define_node {
    ($(#[$attr:meta])* $name:ident) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $(#[$attr])*
        pub struct $name {
            pub(crate) syntax: SyntaxNode,
        }

        impl AstNode for $name {
            fn cast(syntax: SyntaxNode) -> Option<Self> {
                if syntax.kind() == SyntaxKind::$name {
                    Some(Self { syntax })
                } else {
                    None
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
        }
    }
}

define_node! {
    Root
}

impl Root {
    pub fn actions(&self) -> AstChildren<Action> {
        self.syntax.cast_children()
    }

    pub fn actions_with_text(&self) -> ActionsWithText {
        ActionsWithText::new(&self.syntax)
    }
}

pub struct ActionsWithText {
    inner: SyntaxElementChildren<YagTemplateLanguage>,
}

impl ActionsWithText {
    pub(crate) fn new(parent: &SyntaxNode) -> Self {
        Self {
            inner: parent.children_with_tokens(),
        }
    }
}

impl Iterator for ActionsWithText {
    type Item = ActionOrText;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find_map(|element| match element {
            SyntaxElement::Node(node) => Action::cast(node).map(ActionOrText::Action),
            SyntaxElement::Token(token) => tokens::Text::cast(token).map(ActionOrText::Text),
        })
    }
}

#[derive(Debug, Clone, Hash)]
pub enum ActionOrText {
    Action(Action),
    Text(tokens::Text),
}

#[derive(Debug, Clone, Hash)]
pub enum Action {
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

impl AstNode for Action {
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        match syntax.kind() {
            SyntaxKind::TemplateDefinition => TemplateDefinition::cast(syntax).map(Self::TemplateDefinition),
            SyntaxKind::TemplateBlock => TemplateBlock::cast(syntax).map(Self::TemplateBlock),
            SyntaxKind::TemplateInvocation => TemplateInvocation::cast(syntax).map(Self::TemplateInvocation),
            SyntaxKind::Return => ReturnAction::cast(syntax).map(Self::Return),
            SyntaxKind::IfConditional => IfConditional::cast(syntax).map(Self::IfConditional),
            SyntaxKind::WithConditional => WithConditional::cast(syntax).map(Self::WithConditional),
            SyntaxKind::RangeLoop => RangeLoop::cast(syntax).map(Self::RangeLoop),
            SyntaxKind::WhileLoop => WhileLoop::cast(syntax).map(Self::WhileLoop),
            SyntaxKind::LoopBreak => LoopBreak::cast(syntax).map(Self::LoopBreak),
            SyntaxKind::LoopContinue => LoopContinue::cast(syntax).map(Self::LoopContinue),
            SyntaxKind::TryCatchAction => TryCatchAction::cast(syntax).map(Self::TryCatch),
            SyntaxKind::ExprAction => ExprAction::cast(syntax).map(Self::ExprAction),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::TemplateDefinition(v) => &v.syntax,
            Self::TemplateBlock(v) => &v.syntax,
            Self::TemplateInvocation(v) => &v.syntax,
            Self::Return(v) => &v.syntax,
            Self::IfConditional(v) => &v.syntax,
            Self::WithConditional(v) => &v.syntax,
            Self::RangeLoop(v) => &v.syntax,
            Self::WhileLoop(v) => &v.syntax,
            Self::LoopBreak(v) => &v.syntax,
            Self::LoopContinue(v) => &v.syntax,
            Self::TryCatch(v) => &v.syntax,
            Self::ExprAction(v) => &v.syntax,
        }
    }
}

macro_rules! delim_accessors {
    ($name:ident) => {
        impl $name {
            pub fn left_delim(&self) -> Option<tokens::LeftDelim> {
                self.syntax.find_first_token()
            }

            pub fn right_delim(&self) -> Option<tokens::RightDelim> {
                self.syntax.find_last_token()
            }
        }
    };
}

define_node! {
    ActionList
}

impl ActionList {
    pub fn actions(&self) -> AstChildren<Action> {
        self.syntax.cast_children()
    }

    pub fn actions_with_text(&self) -> ActionsWithText {
        ActionsWithText::new(&self.syntax)
    }
}

define_node! {
    TemplateDefinition
}

impl TemplateDefinition {
    pub fn define_clause(&self) -> Option<DefineClause> {
        self.syntax.find_first_child()
    }

    pub fn action_list(&self) -> Option<ActionList> {
        self.syntax.find_first_child()
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        self.syntax.find_last_child()
    }
}

define_node! {
    DefineClause
}
delim_accessors!(DefineClause);

impl DefineClause {
    pub fn template_name(&self) -> Option<tokens::StringLiteral> {
        self.syntax.find_first_token()
    }
}

define_node! {
    TemplateBlock
}

impl TemplateBlock {
    pub fn block_clause(&self) -> Option<BlockClause> {
        self.syntax.find_first_child()
    }

    pub fn action_list(&self) -> Option<ActionList> {
        self.syntax.find_first_child()
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        self.syntax.find_last_child()
    }
}

define_node! {
    BlockClause
}
delim_accessors!(BlockClause);

impl BlockClause {
    pub fn template_name(&self) -> Option<tokens::StringLiteral> {
        self.syntax.find_first_token()
    }

    pub fn context_expr(&self) -> Option<Expr> {
        self.syntax.find_last_child()
    }
}

define_node! {
    TemplateInvocation
}
delim_accessors!(TemplateInvocation);

impl TemplateInvocation {
    pub fn template_name(&self) -> Option<tokens::StringLiteral> {
        self.syntax.find_first_token()
    }

    pub fn context_expr(&self) -> Option<Expr> {
        self.syntax.find_last_child()
    }
}

define_node! {
    ReturnAction
}
delim_accessors!(ReturnAction);

impl ReturnAction {
    pub fn return_expr(&self) -> Option<Expr> {
        self.syntax.find_first_child()
    }
}

define_node! {
    EndClause
}
delim_accessors!(EndClause);

define_node! {
    IfConditional
}

impl IfConditional {
    pub fn if_clause(&self) -> Option<IfClause> {
        self.syntax.find_first_child()
    }

    pub fn if_action_list(&self) -> Option<ActionList> {
        self.syntax.find_first_child()
    }

    pub fn else_branches(&self) -> AstChildren<ElseBranch> {
        self.syntax.cast_children()
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        self.syntax.find_last_child()
    }
}

define_node! {
    IfClause
}
delim_accessors!(IfClause);

impl IfClause {
    pub fn if_expr(&self) -> Option<Expr> {
        self.syntax.find_first_child()
    }
}

define_node! {
    WithConditional
}

impl WithConditional {
    pub fn with_clause(&self) -> Option<WithClause> {
        self.syntax.find_first_child()
    }

    pub fn with_action_list(&self) -> Option<ActionList> {
        self.syntax.find_first_child()
    }

    pub fn else_branches(&self) -> AstChildren<ElseBranch> {
        self.syntax.cast_children()
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        self.syntax.find_last_child()
    }
}

define_node! {
    WithClause
}
delim_accessors!(WithClause);

impl WithClause {
    pub fn with_expr(&self) -> Option<Expr> {
        self.syntax.find_first_child()
    }
}

define_node! {
    ElseBranch
}

impl ElseBranch {
    pub fn else_clause(&self) -> Option<ElseClause> {
        self.syntax.find_first_child()
    }

    pub fn action_list(&self) -> Option<ActionList> {
        self.syntax.find_first_child()
    }
}

define_node! {
    ElseClause
}
delim_accessors!(ElseClause);

impl ElseClause {
    pub fn cond_expr(&self) -> Option<Expr> {
        self.syntax.find_first_child()
    }
}

define_node! {
    RangeLoop
}

impl RangeLoop {
    pub fn range_clause(&self) -> Option<RangeClause> {
        self.syntax.find_first_child()
    }

    pub fn action_list(&self) -> Option<ActionList> {
        self.syntax.find_first_child()
    }

    pub fn else_branch(&self) -> Option<ElseBranch> {
        self.syntax.find_first_child()
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        self.syntax.find_last_child()
    }
}

define_node! {
    RangeClause
}
delim_accessors!(RangeClause);

impl RangeClause {
    pub fn iteration_vars(&self) -> AstTokenChildren<tokens::Var> {
        self.syntax.cast_tokens()
    }

    pub fn range_expr(&self) -> Option<Expr> {
        self.syntax.find_first_child()
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
    WhileLoop
}

impl WhileLoop {
    pub fn while_clause(&self) -> Option<WhileClause> {
        self.syntax.find_first_child()
    }

    pub fn action_list(&self) -> Option<ActionList> {
        self.syntax.find_first_child()
    }

    pub fn else_branch(&self) -> Option<ElseBranch> {
        self.syntax.find_first_child()
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        self.syntax.find_first_child()
    }
}

define_node! {
    WhileClause
}
delim_accessors!(WhileClause);

impl WhileClause {
    pub fn cond_expr(&self) -> Option<Expr> {
        self.syntax.find_first_child()
    }
}

define_node! {
    LoopBreak
}
delim_accessors!(LoopBreak);

define_node! {
    LoopContinue
}
delim_accessors!(LoopContinue);

define_node! {
    TryCatchAction
}

impl TryCatchAction {
    pub fn try_clause(&self) -> Option<TryClause> {
        self.syntax.find_first_child()
    }

    pub fn try_action_list(&self) -> Option<ActionList> {
        self.syntax.find_first_child()
    }

    pub fn catch_clause(&self) -> Option<CatchClause> {
        self.syntax.find_first_child()
    }

    pub fn catch_action_list(&self) -> Option<ActionList> {
        self.syntax.cast_children().nth(1)
    }
}

define_node! {
    TryClause
}
delim_accessors!(TryClause);

define_node! {
    CatchClause
}
delim_accessors!(CatchClause);

define_node! {
    ExprAction
}
delim_accessors!(ExprAction);

impl ExprAction {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.find_first_child()
    }
}

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
    Literal(Literal),
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
            SyntaxKind::Literal => Literal::cast(node).map(Self::Literal),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::FuncCall(v) => &v.syntax,
            Self::ExprCall(v) => &v.syntax,
            Self::Parenthesized(v) => &v.syntax,
            Self::Pipeline(v) => &v.syntax,
            Self::ContextAccess(v) => &v.syntax,
            Self::ContextFieldChain(v) => &v.syntax,
            Self::ExprFieldChain(v) => &v.syntax,
            Self::VarAccess(v) => &v.syntax,
            Self::VarDecl(v) => &v.syntax,
            Self::VarAssign(v) => &v.syntax,
            Self::Literal(v) => &v.syntax,
        }
    }
}

define_node! {
    FuncCall
}

impl FuncCall {
    pub fn func_name(&self) -> Option<tokens::Ident> {
        self.syntax.find_first_token()
    }

    pub fn call_args(&self) -> AstChildren<Expr> {
        self.syntax.cast_children()
    }
}

define_node! {
    ExprCall
}

impl ExprCall {
    pub fn callee(&self) -> Option<Expr> {
        self.syntax.find_first_child()
    }

    pub fn call_args(&self) -> Skip<AstChildren<Expr>> {
        self.syntax
            .cast_children()
            .skip(if self.callee().is_some() { 1 } else { 0 })
    }
}

define_node! {
    ParenthesizedExpr
}

impl ParenthesizedExpr {
    pub fn inner_expr(&self) -> Option<Expr> {
        self.syntax.find_first_child()
    }
}

define_node! {
    Pipeline
}

impl Pipeline {
    pub fn init_expr(&self) -> Option<Expr> {
        self.syntax.find_first_child()
    }

    pub fn stages(&self) -> AstChildren<PipelineStage> {
        self.syntax.cast_children()
    }
}

define_node! {
    PipelineStage
}

impl PipelineStage {
    pub fn target_expr(&self) -> Option<Expr> {
        self.syntax.find_first_child()
    }
}

define_node! {
    ContextAccess
}

define_node! {
    ContextFieldChain
}

impl ContextFieldChain {
    pub fn fields(&self) -> AstTokenChildren<tokens::Field> {
        self.syntax.cast_tokens()
    }
}

define_node! {
    ExprFieldChain
}

impl ExprFieldChain {
    pub fn fields(&self) -> AstTokenChildren<tokens::Field> {
        self.syntax.cast_tokens()
    }
}

define_node! {
    VarAccess
}

impl VarAccess {
    pub fn var(&self) -> Option<tokens::Var> {
        self.syntax.find_first_token()
    }
}

define_node! {
    VarDecl
}

impl VarDecl {
    pub fn var(&self) -> Option<tokens::Var> {
        self.syntax.find_first_token()
    }

    pub fn initializer(&self) -> Option<Expr> {
        self.syntax.find_last_child()
    }
}

define_node! {
    VarAssign
}

impl VarAssign {
    pub fn var(&self) -> Option<tokens::Var> {
        self.syntax.find_first_token()
    }

    pub fn assign_expr(&self) -> Option<Expr> {
        self.syntax.find_last_child()
    }
}

define_node! {
    Literal
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum LiteralKind {
    String(tokens::StringLiteral),
    Bool(tokens::Bool),
    Int(tokens::Int),
    Float(tokens::Float),
    Char(tokens::Char),
    Nil(tokens::Nil),
}

impl Literal {
    pub fn kind(&self) -> LiteralKind {
        let token = self
            .syntax
            .children_with_tokens()
            .find(|element| !element.kind().is_trivia())
            .and_then(|element| element.into_token())
            .expect("literal node should contain token");
        if let Some(t) = tokens::StringLiteral::cast(token.clone()) {
            LiteralKind::String(t)
        } else if let Some(t) = tokens::Bool::cast(token.clone()) {
            LiteralKind::Bool(t)
        } else if let Some(t) = tokens::Int::cast(token.clone()) {
            LiteralKind::Int(t)
        } else if let Some(t) = tokens::Float::cast(token.clone()) {
            LiteralKind::Float(t)
        } else if let Some(t) = tokens::Char::cast(token.clone()) {
            LiteralKind::Char(t)
        } else if let Some(t) = tokens::Nil::cast(token.clone()) {
            LiteralKind::Nil(t)
        } else {
            panic!("unknown token in literal: {token}")
        }
    }
}
