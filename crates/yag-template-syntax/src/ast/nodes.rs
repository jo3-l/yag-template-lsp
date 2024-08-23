use std::iter::Skip;

use rowan::TextSize;

use super::ext::{AstChildren, AstTokenChildren, SyntaxNodeExt};
use super::macros::{define_ast_enum, define_ast_node, define_delim_accessors};
use crate::ast::{tokens, AstNode, AstToken, SyntaxElementChildren};
use crate::{SyntaxElement, SyntaxKind, SyntaxNode, YagTemplateLanguage};

define_ast_node! {
    pub struct Root;
}

impl Root {
    pub fn actions(&self) -> AstChildren<Action> {
        self.syntax.matching_children()
    }

    pub fn actions_with_text(&self) -> ActionsWithText {
        ActionsWithText::new(&self.syntax)
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ActionOrText {
    Action(Action),
    Text(tokens::Text),
}

define_ast_enum! {
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
}

define_ast_node! {
    pub struct ActionList;
}

impl ActionList {
    pub fn actions(&self) -> AstChildren<Action> {
        self.syntax.matching_children()
    }

    pub fn actions_with_text(&self) -> ActionsWithText {
        ActionsWithText::new(&self.syntax)
    }

    pub fn trimmed_end_pos(&self) -> TextSize {
        let mut pos = self.syntax.text_range().end();
        if let Some(ActionOrText::Text(trailing_text)) = self.actions_with_text().last() {
            let trailing_text = trailing_text.get();
            let trailing_spaces = trailing_text.len() - trailing_text.trim_end().len();
            pos -= TextSize::from(trailing_spaces as u32);
        }
        pos
    }
}

define_ast_node! {
    pub struct TemplateDefinition;
}

impl TemplateDefinition {
    pub fn clause(&self) -> Option<DefineClause> {
        self.syntax.first_matching_child()
    }

    pub fn template_body(&self) -> Option<ActionList> {
        self.syntax.first_matching_child()
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        self.syntax.last_matching_child()
    }
}

define_ast_node! {
    pub struct DefineClause;
}
define_delim_accessors!(DefineClause);

impl DefineClause {
    pub fn template_name(&self) -> Option<tokens::StringLiteral> {
        self.syntax.first_matching_token()
    }
}

define_ast_node! {
    pub struct TemplateBlock;
}

impl TemplateBlock {
    pub fn clause(&self) -> Option<BlockClause> {
        self.syntax.first_matching_child()
    }

    pub fn template_body(&self) -> Option<ActionList> {
        self.syntax.first_matching_child()
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        self.syntax.last_matching_child()
    }
}

define_ast_node! {
    pub struct BlockClause;
}
define_delim_accessors!(BlockClause);

impl BlockClause {
    pub fn template_name(&self) -> Option<tokens::StringLiteral> {
        self.syntax.first_matching_token()
    }

    pub fn context_data(&self) -> Option<Expr> {
        self.syntax.last_matching_child()
    }
}

define_ast_node! {
    pub struct TemplateInvocation;
}
define_delim_accessors!(TemplateInvocation);

impl TemplateInvocation {
    pub fn template_name(&self) -> Option<tokens::StringLiteral> {
        self.syntax.first_matching_token()
    }

    pub fn context_data(&self) -> Option<Expr> {
        self.syntax.last_matching_child()
    }
}

define_ast_node! {
    pub struct ReturnAction;
}
define_delim_accessors!(ReturnAction);

impl ReturnAction {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.first_matching_child()
    }
}

define_ast_node! {
    pub struct EndClause;
}
define_delim_accessors!(EndClause);

define_ast_node! {
    pub struct IfConditional;
}

impl IfConditional {
    pub fn clause(&self) -> Option<IfClause> {
        self.syntax.first_matching_child()
    }

    pub fn body(&self) -> Option<ActionList> {
        self.syntax.first_matching_child()
    }

    pub fn else_branches(&self) -> AstChildren<ElseBranch> {
        self.syntax.matching_children()
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        self.syntax.last_matching_child()
    }
}

define_ast_node! {
    pub struct IfClause;
}
define_delim_accessors!(IfClause);

impl IfClause {
    pub fn condition(&self) -> Option<Expr> {
        self.syntax.first_matching_child()
    }
}

define_ast_node! {
    pub struct WithConditional;
}

impl WithConditional {
    pub fn clause(&self) -> Option<WithClause> {
        self.syntax.first_matching_child()
    }

    pub fn body(&self) -> Option<ActionList> {
        self.syntax.first_matching_child()
    }

    pub fn else_branches(&self) -> AstChildren<ElseBranch> {
        self.syntax.matching_children()
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        self.syntax.last_matching_child()
    }
}

define_ast_node! {
    pub struct WithClause;
}
define_delim_accessors!(WithClause);

impl WithClause {
    pub fn condition(&self) -> Option<Expr> {
        self.syntax.first_matching_child()
    }
}

define_ast_node! {
    pub struct ElseBranch;
}

impl ElseBranch {
    pub fn clause(&self) -> Option<ElseClause> {
        self.syntax.first_matching_child()
    }

    pub fn body(&self) -> Option<ActionList> {
        self.syntax.first_matching_child()
    }
}

define_ast_node! {
    pub struct ElseClause;
}
define_delim_accessors!(ElseClause);

impl ElseClause {
    pub fn condition(&self) -> Option<Expr> {
        self.syntax.first_matching_child()
    }
}

define_ast_node! {
    pub struct RangeLoop;
}

impl RangeLoop {
    pub fn clause(&self) -> Option<RangeClause> {
        self.syntax.first_matching_child()
    }

    pub fn body(&self) -> Option<ActionList> {
        self.syntax.first_matching_child()
    }

    pub fn else_branch(&self) -> Option<ElseBranch> {
        self.syntax.first_matching_child()
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        self.syntax.last_matching_child()
    }
}

define_ast_node! {
    pub struct RangeClause;
}
define_delim_accessors!(RangeClause);

impl RangeClause {
    pub fn iteration_vars(&self) -> AstTokenChildren<tokens::Var> {
        self.syntax.matching_tokens()
    }

    pub fn expr(&self) -> Option<Expr> {
        self.syntax.first_matching_child()
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

define_ast_node! {
    pub struct WhileLoop;
}

impl WhileLoop {
    pub fn clause(&self) -> Option<WhileClause> {
        self.syntax.first_matching_child()
    }

    pub fn actions(&self) -> Option<ActionList> {
        self.syntax.first_matching_child()
    }

    pub fn else_branch(&self) -> Option<ElseBranch> {
        self.syntax.first_matching_child()
    }

    pub fn end_clause(&self) -> Option<EndClause> {
        self.syntax.first_matching_child()
    }
}

define_ast_node! {
    pub struct WhileClause;
}
define_delim_accessors!(WhileClause);

impl WhileClause {
    pub fn condition(&self) -> Option<Expr> {
        self.syntax.first_matching_child()
    }
}

define_ast_node! {
    pub struct LoopBreak;
}
define_delim_accessors!(LoopBreak);

define_ast_node! {
    pub struct LoopContinue;
}
define_delim_accessors!(LoopContinue);

define_ast_node! {
    pub struct TryCatchAction;
}

impl TryCatchAction {
    pub fn try_clause(&self) -> Option<TryClause> {
        self.syntax.first_matching_child()
    }

    pub fn try_body(&self) -> Option<ActionList> {
        self.syntax.first_matching_child()
    }

    pub fn catch_clause(&self) -> Option<CatchClause> {
        self.syntax.first_matching_child()
    }

    pub fn catch_body(&self) -> Option<ActionList> {
        self.syntax.matching_children().nth(1)
    }
}

define_ast_node! {
    pub struct TryClause;
}
define_delim_accessors!(TryClause);

define_ast_node! {
    pub struct CatchClause;
}
define_delim_accessors!(CatchClause);

define_ast_node! {
    pub struct ExprAction;
}
define_delim_accessors!(ExprAction);

impl ExprAction {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax.first_matching_child()
    }
}

define_ast_enum! {
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
}

define_ast_node! {
    pub struct FuncCall;
}

impl FuncCall {
    pub fn func_name(&self) -> Option<tokens::Ident> {
        self.syntax.first_matching_token()
    }

    pub fn args(&self) -> AstChildren<Expr> {
        self.syntax.matching_children()
    }
}

define_ast_node! {
    pub struct ExprCall;
}

impl ExprCall {
    pub fn callee(&self) -> Option<Expr> {
        self.syntax.first_matching_child()
    }

    pub fn args(&self) -> Skip<AstChildren<Expr>> {
        self.syntax
            .matching_children()
            .skip(if self.callee().is_some() { 1 } else { 0 })
    }
}

define_ast_node! {
    pub struct ParenthesizedExpr;
}

impl ParenthesizedExpr {
    pub fn inner_expr(&self) -> Option<Expr> {
        self.syntax.first_matching_child()
    }
}

define_ast_node! {
    pub struct Pipeline;
}

impl Pipeline {
    pub fn init_expr(&self) -> Option<Expr> {
        self.syntax.first_matching_child()
    }

    pub fn stages(&self) -> AstChildren<PipelineStage> {
        self.syntax.matching_children()
    }
}

define_ast_node! {
    pub struct PipelineStage;
}

impl PipelineStage {
    pub fn call_expr(&self) -> Option<Expr> {
        self.syntax.first_matching_child()
    }
}

define_ast_node! {
    pub struct ContextAccess;
}

define_ast_node! {
    pub struct ContextFieldChain;
}

impl ContextFieldChain {
    pub fn fields(&self) -> AstTokenChildren<tokens::Field> {
        self.syntax.matching_tokens()
    }
}

define_ast_node! {
    pub struct ExprFieldChain;
}

impl ExprFieldChain {
    pub fn base_expr(&self) -> Option<Expr> {
        self.syntax.first_matching_child()
    }

    pub fn fields(&self) -> AstTokenChildren<tokens::Field> {
        self.syntax.matching_tokens()
    }
}

define_ast_node! {
    pub struct VarAccess;
}

impl VarAccess {
    pub fn var(&self) -> Option<tokens::Var> {
        self.syntax.first_matching_token()
    }
}

define_ast_node! {
    pub struct VarDecl;
}

impl VarDecl {
    pub fn var(&self) -> Option<tokens::Var> {
        self.syntax.first_matching_token()
    }

    pub fn initializer(&self) -> Option<Expr> {
        self.syntax.last_matching_child()
    }
}

define_ast_node! {
    pub struct VarAssign;
}

impl VarAssign {
    pub fn var(&self) -> Option<tokens::Var> {
        self.syntax.first_matching_token()
    }

    pub fn assign_expr(&self) -> Option<Expr> {
        self.syntax.last_matching_child()
    }
}

define_ast_node! {
    pub struct Literal;
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
            unreachable!("unknown token in literal: {token}")
        }
    }
}
