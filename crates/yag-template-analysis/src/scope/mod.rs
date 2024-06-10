mod analysis;
mod info;

pub use analysis::analyze;
pub use info::{ParentScopesIter, Scope, ScopeId, ScopeInfo};
