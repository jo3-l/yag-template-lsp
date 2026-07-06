use yag_template_envdefs::EnvDefs;
use yag_template_syntax::ast;
use yag_template_syntax::ast::{AstNode, AstToken};

use crate::AnalysisWarning;

pub fn check(env: &EnvDefs, root: &ast::Root) -> Vec<AnalysisWarning> {
    root.syntax()
        .descendants()
        .filter_map(ast::FuncCall::cast)
        .filter_map(|call| check_func_call(env, call))
        .collect()
}

fn check_func_call(env: &EnvDefs, call: ast::FuncCall) -> Option<AnalysisWarning> {
    let func_name_ident = call.func_name()?;
    let func_name = func_name_ident.get();
    let Some(func) = env.funcs.get(func_name) else {
        return None;
    };
    func.is_deprecated
        .then(|| AnalysisWarning::new(format!("{func_name} is deprecated"), func_name_ident.text_range(), true))
}
