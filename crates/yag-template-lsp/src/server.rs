use std::sync::Arc;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{async_trait, Client, LanguageServer};

use crate::session::Session;
use crate::{provider, session};

pub(super) struct YagTemplateLanguageServer {
    client: Client,
    session: Arc<Session>,
}

impl YagTemplateLanguageServer {
    pub(super) fn new(client: Client) -> Self {
        Self {
            client: client.clone(),
            session: Arc::new(Session::new(client)),
        }
    }
}

#[async_trait]
impl LanguageServer for YagTemplateLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "YAGPDB Template Language Server".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        session::sync::on_document_open(&self.session, params).await.unwrap()
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        session::sync::on_document_change(&self.session, params).await.unwrap()
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        session::sync::on_document_close(&self.session, params).await;
    }
}
