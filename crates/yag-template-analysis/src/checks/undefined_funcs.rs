use yag_template_envdefs::EnvDefs;
use yag_template_syntax::ast;
use yag_template_syntax::ast::{AstNode, AstToken};

use crate::AnalysisError;

pub fn check(env: &EnvDefs, root: ast::Root) -> Vec<AnalysisError> {
    root.syntax()
        .descendants()
        .filter_map(ast::FuncCall::cast)
        .filter_map(|call| check_func_call(env, call))
        .collect()
}

fn check_func_call(env: &EnvDefs, call: ast::FuncCall) -> Option<AnalysisError> {
    let func_name_ident = call.func_name()?;
    let func_name = func_name_ident.get();
    if env.funcs.contains_key(func_name) {
        None
    } else {
        Some(AnalysisError::new(
            format!("unknown function {func_name}"),
            func_name_ident.text_range(),
        ))
    }
}
