use tower_lsp::lsp_types::{Diagnostic, Url};

use crate::server::YagTemplateLanguageServer;

impl YagTemplateLanguageServer {
    pub(crate) async fn publish_diagnostics(&self, uri: &Url, version: i32) {
        let workspace = self.workspace.read().await;
        let Some(doc) = workspace.document(uri) else {
            return;
        };
        let diags: Vec<Diagnostic> = doc
            .parse
            .errors
            .iter()
            .map(|e| {
                let range = doc.mapper.range(e.range).unwrap_or_default();
                Diagnostic::new_simple(range, e.message.clone())
            })
            .collect();
        self.client
            .publish_diagnostics(uri.clone(), diags, Some(version))
            .await;
    }
}
