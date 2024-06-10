use scope::ScopeInfo;
use yag_template_syntax::ast;

mod scope;
mod typeck;

pub fn analyze(root: ast::Root) -> Analysis {
    Analysis {
        scope_info: scope::analyze(root),
    }
}

pub struct Analysis {
    pub scope_info: ScopeInfo,
}
