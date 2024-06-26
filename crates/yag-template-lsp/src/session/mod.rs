use std::hash::RandomState;

use anyhow::anyhow;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use tower_lsp::lsp_types::Url;
use tower_lsp::Client;

pub(crate) mod document;
pub(crate) mod sync;

pub(crate) use document::Document;

pub(crate) struct Session {
    pub(crate) client: Client,
    documents: DashMap<Url, Document>,
}

impl Session {
    pub(crate) fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
        }
    }

    pub(crate) fn document(&self, uri: &Url) -> anyhow::Result<Ref<'_, Url, Document, RandomState>> {
        self.documents
            .get(uri)
            .ok_or_else(|| anyhow!("could not find document {uri}"))
    }

    pub(crate) fn upsert_document(&self, uri: &Url, document: Document) {
        self.documents.insert(uri.clone(), document);
    }

    pub(crate) fn remove_document(&self, uri: &Url) {
        self.documents.remove(&uri);
    }
}
