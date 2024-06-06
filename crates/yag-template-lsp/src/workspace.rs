use std::collections::HashMap;

use tower_lsp::lsp_types::Url;
use yag_template_syntax::parser::{self, Parse};

use crate::mapper::Mapper;

pub(crate) struct Workspace {
    pub(crate) documents: HashMap<Url, Document>,
}

impl Workspace {
    pub(crate) fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub(crate) fn document(&self, uri: &Url) -> Option<&Document> {
        self.documents.get(&uri)
    }

    pub(crate) fn upsert_document(&mut self, uri: &Url, text: &str) {
        let parse = parser::parse(text);
        let mapper = Mapper::new_utf16(text);
        self.documents
            .insert(uri.clone(), Document { parse, mapper });
    }

    pub(crate) fn remove_document(&mut self, uri: &Url) {
        self.documents.remove(&uri);
    }
}

pub(crate) struct Document {
    pub(crate) parse: Parse,
    pub(crate) mapper: Mapper,
}
