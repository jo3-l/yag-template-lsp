use tower_lsp::lsp_types::{Diagnostic, Url};

use crate::session::Session;

pub(crate) async fn publish_diagnostics(session: &Session, uri: &Url) -> anyhow::Result<()> {
    let doc = session.document(uri)?;
    let diags: Vec<Diagnostic> = doc
        .parse
        .errors
        .iter()
        .map(|e| {
            let range = doc.mapper.range(e.range).unwrap_or_default();
            Diagnostic::new_simple(range, e.message.clone())
        })
        .collect();

    let version = Default::default();
    session
        .client
        .publish_diagnostics(uri.clone(), diags, Some(version))
        .await;
    Ok(())
}
