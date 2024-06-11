use scope::ScopeInfo;
use yag_template_syntax::ast;

pub mod scope;
pub mod typeck;

pub fn analyze(root: ast::Root) -> Analysis {
    Analysis {
        scope_info: scope::analyze(root),
    }
}

pub struct Analysis {
    pub scope_info: ScopeInfo,
}
