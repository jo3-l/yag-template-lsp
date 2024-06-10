use tower_lsp::lsp_types::*;

use crate::provider::diagnostics::publish_diagnostics;
use crate::session::{Document, Session};

pub(crate) async fn on_document_open(session: &Session, params: DidOpenTextDocumentParams) -> anyhow::Result<()> {
    let uri = params.text_document.uri;
    let document = Document::new(&params.text_document.text)?;
    session.upsert_document(&uri, document);
    publish_diagnostics(session, &uri).await
}

pub(crate) async fn on_document_change(session: &Session, params: DidChangeTextDocumentParams) -> anyhow::Result<()> {
    let uri = params.text_document.uri;

    // We're using TextDocumentSyncKind::FULL, so no incremental changes (for now.)
    let document = Document::new(&params.content_changes[0].text)?;
    session.upsert_document(&uri, document);
    publish_diagnostics(session, &uri).await
}

pub(crate) fn on_document_close(session: &Session, params: DidCloseTextDocumentParams) {
    session.remove_document(&params.text_document.uri)
}
