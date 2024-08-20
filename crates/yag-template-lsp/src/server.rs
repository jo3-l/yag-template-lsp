use std::sync::Arc;

use tower_lsp::jsonrpc::{self, Result};
use tower_lsp::lsp_types::{
    CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, FoldingRange, FoldingRangeParams, FoldingRangeProviderCapability, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult,
    InitializedParams, InlayHint, InlayHintParams, Location, OneOf, ReferenceParams, RenameParams, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, WorkspaceEdit,
};
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

macro_rules! try_handle {
    ($resp:expr) => {
        $resp.await.map_err(|_| jsonrpc::Error::internal_error())
    };
}

#[async_trait]
impl LanguageServer for YagTemplateLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let extra_completion_trigger_chars = vec!['$'];
        let completion_trigger_chars: Vec<_> = ('a'..='z')
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
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "YAGPDB Template Language Server".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("server initialized")
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        session::sync::on_document_open(&self.session, params).await.unwrap();
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        session::sync::on_document_change(&self.session, params).await.unwrap();
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        session::sync::on_document_close(&self.session, params).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        try_handle!(provider::completion::complete(&self.session, params))
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        try_handle!(provider::folding_range::folding_range(&self.session, params))
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        try_handle!(provider::goto_definition::goto_definition(&self.session, params))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        try_handle!(provider::hover::hover(&self.session, params))
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        try_handle!(provider::inlay_hint::inlay_hint(&self.session, params))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        try_handle!(provider::references::references(&self.session, params))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        try_handle!(provider::rename::rename(&self.session, params))
    }
}
