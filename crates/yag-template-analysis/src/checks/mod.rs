use yag_template_envdefs::EnvDefs;
use yag_template_syntax::ast;

use crate::AnalysisError;

pub mod undefined_funcs;

pub fn run_all(env: &EnvDefs, root: ast::Root) -> Vec<AnalysisError> {
    undefined_funcs::check(env, root)
}
