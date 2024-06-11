use std::sync::Arc;

use tower_lsp::jsonrpc::{self, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{async_trait, Client, LanguageServer};

use crate::provider;
use crate::session::{self, Session};

pub(super) struct YagTemplateLanguageServer {
    session: Arc<Session>,
}

impl YagTemplateLanguageServer {
    pub(super) fn new(client: Client) -> Self {
        Self {
            session: Arc::new(Session::new(client)),
        }
    }
}

#[async_trait]
impl LanguageServer for YagTemplateLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let extra_completion_trigger_chars = vec!['.'];
        let completion_trigger_chars: Vec<String> = ('a'..='z')
            .chain('A'..='Z')
            .chain(extra_completion_trigger_chars)
            .map(|c| c.to_string())
            .collect();

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(completion_trigger_chars),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        provider::completion::complete(&self.session, params)
            .await
            .map_err(|_| jsonrpc::Error::internal_error())
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        provider::hover::hover(&self.session, params)
            .await
            .map_err(|_| jsonrpc::Error::internal_error())
    }
}
