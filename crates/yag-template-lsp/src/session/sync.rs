use tower_lsp::lsp_types::{DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams};

use crate::provider;
use crate::session::{Document, Session};

pub(crate) async fn on_document_open(sess: &Session, params: DidOpenTextDocumentParams) -> anyhow::Result<()> {
    let uri = params.text_document.uri;
    let document = Document::new(sess, &params.text_document.text)?;
    sess.upsert_document(&uri, document);
    provider::diagnostics::publish_diagnostics(sess, &uri).await
}

pub(crate) async fn on_document_change(sess: &Session, params: DidChangeTextDocumentParams) -> anyhow::Result<()> {
    let uri = params.text_document.uri;

    // We're using TextDocumentSyncKind::FULL, so no incremental changes (for now.)
    let document = Document::new(sess, &params.content_changes[0].text)?;
    sess.upsert_document(&uri, document);
    provider::diagnostics::publish_diagnostics(sess, &uri).await
}

pub(crate) async fn on_document_close(sess: &Session, params: DidCloseTextDocumentParams) {
    let uri = params.text_document.uri;
    sess.remove_document(&uri);
    provider::diagnostics::clear_diagnostics(sess, &uri).await;
}
