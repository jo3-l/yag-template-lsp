use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{async_trait, Client, LanguageServer};

use crate::workspace::Workspace;

pub(crate) struct YagTemplateLanguageServer {
    pub(crate) client: Client,
    pub(crate) workspace: RwLock<Workspace>,
}

impl YagTemplateLanguageServer {
    pub(crate) fn new(client: Client) -> Self {
        Self {
            client,
            workspace: RwLock::new(Workspace::new()),
        }
    }
}

#[async_trait]
impl LanguageServer for YagTemplateLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
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
        let doc = params.text_document;
        self.on_change(&doc.uri, &doc.text, doc.version).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.on_change(
            &params.text_document.uri,
            // no incremental changes for now (we use
            // TextDocumentSyncKind::FULL)
            &params.content_changes[0].text,
            params.text_document.version,
        )
        .await;
    }
}

impl YagTemplateLanguageServer {
    pub(crate) async fn on_change(&self, uri: &Url, text: &str, version: i32) {
        self.workspace.write().await.upsert_document(uri, text);
        self.publish_diagnostics(uri, version).await;
    }
}
