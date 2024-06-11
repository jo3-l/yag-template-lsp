use slotmap::SlotMap;
use smol_str::SmolStr;
use yag_template_syntax::ast::{self, Action, AstNode, AstToken, SyntaxNodeExt};
use yag_template_syntax::{SyntaxNode, TextRange, TextSize};

use crate::scope::info::{Scope, ScopeId, ScopeInfo, Var};

pub fn analyze(root: ast::Root) -> ScopeInfo {
    let mut s = ScopeAnalyzer::new();
    s.enter_inner_scope(root.syntax().text_range());
    // The variable $ is predefined as the initial context data.
    s.push_synthetic_var("$", 0.into());
    s.analyze_all(root.actions());
    s.exit_scope();
    s.finish()
}

struct ScopeAnalyzer {
    scopes: SlotMap<ScopeId, Scope>,
    stack: Vec<ScopeId>,
    pending_vars: Vec<Var>,
}

impl ScopeAnalyzer {
    fn new() -> Self {
        Self {
            scopes: SlotMap::with_key(),
            stack: Vec::new(),
            pending_vars: Vec::new(),
        }
    }

    fn finish(self) -> ScopeInfo {
        debug_assert!(self.pending_vars.is_empty());
        debug_assert!(self.stack.is_empty());
        ScopeInfo::new(self.scopes)
    }

    fn analyze_all(&mut self, actions: impl Iterator<Item = Action>) {
        for action in actions {
            self.analyze_action(action);
        }
    }

    fn analyze_action(&mut self, action: ast::Action) {
        match action {
            Action::TemplateDefinition(template_def) => self.analyze_template_def(template_def),
            Action::TemplateBlock(block) => self.analyze_template_block(block),
            Action::TemplateInvocation(invocation) => self.analyze_template_invocation(invocation),
            Action::Return(return_action) => self.analyze_return_action(return_action),
            Action::IfConditional(if_conditional) => self.analyze_if_conditional(if_conditional),
            Action::WithConditional(with_conditional) => self.analyze_with_conditional(with_conditional),
            Action::RangeLoop(range_loop) => self.analyze_range_loop(range_loop),
            Action::WhileLoop(while_loop) => self.analyze_while_loop(while_loop),
            Action::LoopBreak(_) | Action::LoopContinue(_) => {}
            Action::TryCatch(try_catch) => self.analyze_try_catch_action(try_catch),
            Action::ExprAction(expr_action) => self.analyze_expr_action(expr_action),
        }
    }

    fn analyze_template_def(&mut self, def: ast::TemplateDefinition) {
        if let Some(list) = def.action_list() {
            self.enter_detached_scope(list.syntax().text_range());
            // All associated template executions have the variable $ predefined
            // as the initial context data.
            self.push_synthetic_var("$", list.syntax().text_range().start());
            self.analyze_all(list.actions());
            self.exit_scope();
        }
    }

    fn analyze_template_block(&mut self, block: ast::TemplateBlock) {
        self.push_var_decls_in(|| block.block_clause()?.context_expr());

        if let Some(list) = block.action_list() {
            self.enter_detached_scope(list.syntax().text_range());
            self.push_synthetic_var("$", list.syntax().text_range().start());
            self.analyze_all(list.actions());
            self.exit_scope();
        }
    }

    fn analyze_template_invocation(&mut self, invocation: ast::TemplateInvocation) {
        self.push_var_decls_in(|| invocation.context_expr());
    }

    fn analyze_return_action(&mut self, return_action: ast::ReturnAction) {
        self.push_var_decls_in(|| return_action.return_expr());
    }

    fn analyze_if_conditional(&mut self, if_conditional: ast::IfConditional) {
        let Some(if_clause) = if_conditional.if_clause() else {
            return;
        };
        let if_clause_scope = self.enter_inner_scope(if_clause.syntax().text_range());
        self.push_var_decls_in(|| if_clause.if_expr());
        self.exit_scope();
        if let Some(if_list) = if_conditional.if_action_list() {
            self.enter_scope_with_parent(if_list.syntax().text_range(), if_clause_scope);
            self.analyze_all(if_list.actions());
            self.exit_scope();
        }
        self.analyze_if_or_with_else_branches(if_clause_scope, if_conditional.else_branches());
    }

    fn analyze_with_conditional(&mut self, with_conditional: ast::WithConditional) {
        let Some(with_clause) = with_conditional.with_clause() else {
            return;
        };
        let with_clause_scope = self.enter_inner_scope(with_clause.syntax().text_range());
        self.push_var_decls_in(|| with_clause.with_expr());
        self.exit_scope();
        if let Some(with_list) = with_conditional.with_action_list() {
            self.enter_scope_with_parent(with_list.syntax().text_range(), with_clause_scope);
            self.analyze_all(with_list.actions());
            self.exit_scope();
        }
        self.analyze_if_or_with_else_branches(with_clause_scope, with_conditional.else_branches());
    }

    fn analyze_if_or_with_else_branches(
        &mut self,
        if_or_with_clause_scope: ScopeId,
        else_branches: impl Iterator<Item = ast::ElseBranch>,
    ) {
        let mut parent_scope = if_or_with_clause_scope;
        for else_branch in else_branches {
            if let Some(else_clause) = else_branch.else_clause() {
                parent_scope = self.enter_scope_with_parent(else_clause.syntax().text_range(), parent_scope);
            }

            if let Some(else_list) = else_branch.action_list() {
                self.enter_scope_with_parent(else_list.syntax().text_range(), parent_scope);
                self.analyze_all(else_list.actions());
                self.exit_scope();
            }
        }
    }

    fn analyze_range_loop(&mut self, range_loop: ast::RangeLoop) {
        let Some(range_clause) = range_loop.range_clause() else {
            return;
        };
        let range_clause_scope = self.enter_inner_scope(range_clause.syntax().text_range());
        if range_clause.declares_vars() {
            self.pending_vars.extend(
                range_clause
                    .iteration_vars()
                    .map(|ast_var| Var::new(ast_var.name(), ast_var.syntax().text_range().end(), None)),
            )
        }
        self.push_var_decls_in(|| range_clause.range_expr());
        self.exit_scope();
        if let Some(range_list) = range_loop.action_list() {
            self.enter_scope_with_parent(range_list.syntax().text_range(), range_clause_scope);
            self.analyze_all(range_list.actions());
            self.exit_scope();
        }
        if let Some(else_branch) = range_loop.else_branch() {
            if let Some(else_list) = else_branch.action_list() {
                self.enter_scope_with_parent(else_list.syntax().text_range(), range_clause_scope);
                self.analyze_all(else_list.actions());
                self.exit_scope();
            }
        }
    }

    fn analyze_while_loop(&mut self, while_loop: ast::WhileLoop) {
        let Some(while_clause) = while_loop.while_clause() else {
            return;
        };
        let while_clause_scope = self.enter_inner_scope(while_clause.syntax().text_range());
        self.push_var_decls_in(|| while_clause.cond_expr());
        self.exit_scope();
        if let Some(while_list) = while_loop.action_list() {
            self.enter_scope_with_parent(while_list.syntax().text_range(), while_clause_scope);
            self.analyze_all(while_list.actions());
            self.exit_scope();
        }
        if let Some(else_branch) = while_loop.else_branch() {
            if let Some(else_list) = else_branch.action_list() {
                self.enter_scope_with_parent(else_list.syntax().text_range(), while_clause_scope);
                self.analyze_all(else_list.actions());
                self.exit_scope();
            }
        }
    }

    fn analyze_try_catch_action(&mut self, try_catch_action: ast::TryCatchAction) {
        if let Some(try_list) = try_catch_action.try_action_list() {
            self.enter_inner_scope(try_list.syntax().text_range());
            self.analyze_all(try_list.actions());
            self.exit_scope();
        }

        if let Some(catch_list) = try_catch_action.catch_action_list() {
            self.enter_inner_scope(catch_list.syntax().text_range());
            self.analyze_all(catch_list.actions());
            self.exit_scope();
        }
    }

    fn analyze_expr_action(&mut self, expr_action: ast::ExprAction) {
        self.push_var_decls_in(|| expr_action.expr());
    }

    fn push_synthetic_var(&mut self, name: impl Into<SmolStr>, visible_from: TextSize) {
        self.pending_vars.push(Var::new(name, visible_from, None));
    }

    fn push_var_decls_in<F>(&mut self, f: F)
    where
        F: FnOnce() -> Option<ast::Expr>,
    {
        if let Some(expr) = f() {
            self.pending_vars.extend(var_decls_in(expr.syntax()));
        }
    }

    fn enter_inner_scope(&mut self, text_range: TextRange) -> ScopeId {
        self.enter_scope(text_range, self.stack.last().copied())
    }

    fn enter_detached_scope(&mut self, text_range: TextRange) -> ScopeId {
        self.enter_scope(text_range, None)
    }

    fn enter_scope_with_parent(&mut self, text_range: TextRange, parent: ScopeId) -> ScopeId {
        self.enter_scope(text_range, Some(parent))
    }

    fn enter_scope(&mut self, text_range: TextRange, parent: Option<ScopeId>) -> ScopeId {
        if let Some(old_scope) = self.stack.last().copied() {
            self.link_pending_vars_to(old_scope);
        }
        let new_scope = self.scopes.insert(Scope::new(text_range, Vec::new(), parent));
        self.stack.push(new_scope);
        new_scope
    }

    fn exit_scope(&mut self) {
        let scope_to_exit = self
            .stack
            .pop()
            .expect("call to exit_scope() should correspond to an earlier enter_scope()");
        self.link_pending_vars_to(scope_to_exit);
    }

    fn link_pending_vars_to(&mut self, to: ScopeId) {
        self.scopes[to].vars.append(&mut self.pending_vars)
    }
}

fn var_decls_in(node: &SyntaxNode) -> impl Iterator<Item = Var> {
    node.descendants()
        .filter_map(|child| child.try_to::<ast::VarDecl>())
        .filter_map(|decl| Var::try_from_decl(decl))
}
