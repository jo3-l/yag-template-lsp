use tower_lsp::lsp_types::{Diagnostic, Url};

use crate::session::Session;

pub(crate) async fn publish_diagnostics(session: &Session, uri: &Url) -> anyhow::Result<()> {
    let doc = session.document(uri)?;
    let diags: Vec<Diagnostic> = doc
        .parse
        .errors
        .iter()
        .map(|e| Diagnostic::new_simple(doc.mapper.range(e.range), e.message.clone()))
        .collect();

    let version = Default::default();
    session
        .client
        .publish_diagnostics(uri.clone(), diags, Some(version))
        .await;
    Ok(())
}

pub(crate) async fn clear_diagnostics(session: &Session, uri: &Url) {
    let version = Default::default();
    session
        .client
        .publish_diagnostics(uri.clone(), Vec::new(), Some(version))
        .await
}
