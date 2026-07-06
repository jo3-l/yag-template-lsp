use core::fmt;
use std::error::Error;

use rowan::TextRange;
use scope::ScopeInfo;
use yag_template_envdefs::EnvDefs;
use yag_template_syntax::ast;

pub mod checks;
pub mod scope;

pub struct Analysis {
    pub scope_info: ScopeInfo,
    pub errors: Vec<AnalysisError>,
    pub warnings: Vec<AnalysisWarning>,
    pub deprecations: Vec<AnalysisWarning>,
}

pub fn analyze(env: &EnvDefs, root: ast::Root) -> Analysis {
    let (scope_info, mut errors, warnings, mut deprecations) = scope::analyze(root.clone());
    errors.extend(checks::undefined_funcs::check(env, &root));
    deprecations.extend(checks::deprecated_funcs::check(env, &root));
    Analysis {
        scope_info,
        errors,
        warnings,
        deprecations,
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisError {
    pub message: String,
    pub range: TextRange,
}

impl AnalysisError {
    pub fn new(message: impl Into<String>, range: TextRange) -> Self {
        Self {
            message: message.into(),
            range,
        }
    }
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl Error for AnalysisError {}

#[derive(Debug, Clone)]
pub struct AnalysisWarning {
    pub message: String,
    pub range: TextRange,
}

impl AnalysisWarning {
    pub fn new(message: impl Into<String>, range: TextRange) -> Self {
        Self {
            message: message.into(),
            range,
        }
    }
}
