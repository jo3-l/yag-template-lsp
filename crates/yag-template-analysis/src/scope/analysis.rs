use std::mem;

use foldhash::{HashMap, HashMapExt};
use rowan::{TextRange, TextSize};
use slotmap::SlotMap;
use smol_str::SmolStr;
use yag_template_syntax::ast;
use yag_template_syntax::ast::{Action, AstNode, AstToken};

use super::{Scope, ScopeId, ScopeInfo, VarSymbol, VarSymbolId};
use crate::AnalysisError;

pub fn analyze(root: ast::Root) -> (ScopeInfo, Vec<AnalysisError>) {
    let root_range = root.syntax().text_range();
    let mut s = ScopeAnalyzer::new(root_range);
    // The variable $ is predefined as the initial context data.
    s.declare_var("$", root_range.start(), None);
    s.analyze_all(root.actions());
    s.finish()
}

struct ScopeAnalyzer {
    scopes: SlotMap<ScopeId, Scope>,
    top_scope: ScopeId,
    scope_stack: Vec<ScopeId>, // lexical parents of top_scope

    var_syms: SlotMap<VarSymbolId, VarSymbol>,
    resolved_var_uses: HashMap<TextRange, VarSymbolId>, // indexed by text range of ast::Var
    errors: Vec<AnalysisError>,
}

impl ScopeAnalyzer {
    fn new(root_range: TextRange) -> Self {
        let mut scopes = SlotMap::with_key();
        let top_scope = scopes.insert(Scope::new(root_range, None));
        Self {
            scopes,
            top_scope,
            scope_stack: Vec::new(),

            var_syms: SlotMap::with_key(),
            resolved_var_uses: HashMap::new(),
            errors: Vec::new(),
        }
    }

    fn finish(self) -> (ScopeInfo, Vec<AnalysisError>) {
        debug_assert!(self.scope_stack.is_empty());
        (
            ScopeInfo::new(self.var_syms, self.resolved_var_uses, self.scopes),
            self.errors,
        )
    }

    fn error(&mut self, message: impl Into<String>, range: TextRange) {
        self.errors.push(AnalysisError::new(message, range));
    }

    fn enter_inner_scope(&mut self, range: TextRange) -> ScopeId {
        self.enter_scope(range, Some(self.top_scope))
    }

    fn enter_detached_scope(&mut self, range: TextRange) -> ScopeId {
        self.enter_scope(range, None)
    }

    fn enter_scope_with_parent(&mut self, range: TextRange, parent: ScopeId) -> ScopeId {
        self.enter_scope(range, Some(parent))
    }

    fn enter_scope(&mut self, range: TextRange, parent: Option<ScopeId>) -> ScopeId {
        let new_scope = self.scopes.insert(Scope::new(range, parent));
        self.scope_stack.push(mem::replace(&mut self.top_scope, new_scope));
        new_scope
    }

    fn exit_scope(&mut self) {
        self.top_scope = self
            .scope_stack
            .pop()
            .expect("call to exit_scope() should correspond to an earlier enter_scope()");
    }

    fn declare_var(
        &mut self,
        var_name: impl Into<SmolStr> + Clone,
        visible_from: TextSize,
        decl_range: Option<TextRange>,
    ) -> VarSymbolId {
        let id = self.var_syms.insert_with_key(|id| VarSymbol {
            id,
            name: var_name.clone().into(),
            visible_from,
            decl_range,
        });
        let top_scope = &mut self.scopes[self.top_scope];
        top_scope.declared_vars.push(self.var_syms[id].clone());
        top_scope.vars_by_name.insert(var_name.into(), id);
        id
    }

    fn set_referent(&mut self, var: ast::Var, sym: VarSymbolId) {
        self.resolved_var_uses.insert(var.syntax().text_range(), sym);
    }
}

macro_rules! access {
    ($e:expr) => {
        || -> Option<_> { $e }()
    };
}

// Action analysis.
impl ScopeAnalyzer {
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
            let body_range = list.syntax().text_range();
            self.enter_detached_scope(body_range);
            // All associated template executions have the variable $ predefined
            // as the initial context data.
            self.declare_var("$", body_range.start(), None);
            self.analyze_all(list.actions());
            self.exit_scope();
        }
    }

    fn analyze_template_block(&mut self, block: ast::TemplateBlock) {
        self.try_analyze_expr(access!(block.block_clause()?.context_expr()));

        if let Some(list) = block.action_list() {
            let body_range = list.syntax().text_range();
            self.enter_detached_scope(body_range);
            self.declare_var("$", body_range.start(), None);
            self.analyze_all(list.actions());
            self.exit_scope();
        }
    }

    fn analyze_template_invocation(&mut self, invocation: ast::TemplateInvocation) {
        self.try_analyze_expr(invocation.context_expr());
    }

    fn analyze_return_action(&mut self, return_action: ast::ReturnAction) {
        self.try_analyze_expr(return_action.return_expr());
    }

    fn analyze_if_conditional(&mut self, if_conditional: ast::IfConditional) {
        let Some(if_clause) = if_conditional.if_clause() else {
            return;
        };
        let if_clause_scope = self.enter_inner_scope(if_clause.syntax().text_range());
        self.try_analyze_expr(if_clause.if_expr());
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
        self.try_analyze_expr(with_clause.with_expr());
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
                self.try_analyze_expr(else_clause.cond_expr());
                self.exit_scope();
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
            for var in range_clause.iteration_vars() {
                let id = self.declare_var(
                    var.name(),
                    range_clause.syntax().text_range().end(),
                    Some(range_clause.syntax().text_range()),
                );
                self.set_referent(var, id);
            }
        } else if range_clause.assigns_vars() {
            for var in range_clause.iteration_vars() {
                self.try_resolve_var_use(var);
            }
        }
        self.try_analyze_expr(range_clause.range_expr());
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
        self.try_analyze_expr(while_clause.cond_expr());
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
        self.try_analyze_expr(expr_action.expr());
    }
}

// Expression analysis.
impl ScopeAnalyzer {
    fn try_analyze_expr(&mut self, expr: Option<ast::Expr>) {
        if let Some(expr) = expr {
            self.analyze_expr(expr);
        }
    }

    fn analyze_expr(&mut self, expr: ast::Expr) {
        use ast::Expr::*;
        match expr {
            FuncCall(call) => call.call_args().for_each(|arg| self.analyze_expr(arg)),
            ExprCall(call) => {
                self.try_analyze_expr(call.callee());
                call.call_args().for_each(|arg| self.analyze_expr(arg));
            }
            Parenthesized(p) => self.try_analyze_expr(p.inner_expr()),
            Pipeline(p) => {
                self.try_analyze_expr(p.init_expr());
                p.stages().for_each(|stage| self.try_analyze_expr(stage.target_expr()));
            }
            ContextAccess(_) => {}
            ContextFieldChain(_) => {}
            ExprFieldChain(chain) => self.try_analyze_expr(chain.base_expr()),
            VarAccess(access) => self.analyze_var_access(access),
            VarDecl(decl) => self.analyze_var_decl(decl),
            VarAssign(assign) => self.analyze_var_assign(assign),
            Literal(_) => {}
        }
    }

    fn analyze_var_access(&mut self, access: ast::VarAccess) {
        if let Some(var) = access.var() {
            self.try_resolve_var_use(var);
        }
    }

    fn analyze_var_decl(&mut self, decl: ast::VarDecl) {
        if let Some(var) = decl.var() {
            let decl_range = decl.syntax().text_range();
            let id = self.declare_var(var.name(), decl_range.end(), Some(decl_range));
            self.set_referent(var, id);
        }
        self.try_analyze_expr(decl.initializer());
    }

    fn analyze_var_assign(&mut self, assign: ast::VarAssign) {
        if let Some(var) = assign.var() {
            self.try_resolve_var_use(var)
        }
        self.try_analyze_expr(assign.assign_expr());
    }

    fn try_resolve_var_use(&mut self, var_use: ast::Var) {
        let name = var_use.name();
        let range = var_use.syntax().text_range();
        match self.lookup_var(name) {
            Some(id) => self.set_referent(var_use, id),
            None => self.error(format!("undefined variable {name}"), range),
        }
    }

    fn lookup_var(&self, name: &str) -> Option<VarSymbolId> {
        let mut cur_scope_id = Some(self.top_scope);
        while let Some(cur_scope) = cur_scope_id.and_then(|id| self.scopes.get(id)) {
            if let Some(id) = cur_scope.vars_by_name.get(name).copied() {
                return Some(id);
            }

            cur_scope_id = cur_scope.parent;
        }
        None
    }
}
