use std::collections::HashMap;

use tower_lsp::lsp_types::{RenameParams, TextEdit, WorkspaceEdit};
use yag_template_syntax::ast;

use crate::session::{Document, Session};

pub(crate) async fn rename(sess: &Session, params: RenameParams) -> anyhow::Result<Option<WorkspaceEdit>> {
    let uri = params.text_document_position.text_document.uri;
    let doc = sess.document(&uri)?;

    let pos = params.text_document_position.position;
    let query = doc.query_at(pos);
    let edits = if let Some(var) = query.var() {
        rename_var(&doc, var, params.new_name)
    } else {
        None
    };
    Ok(edits)
}

fn rename_var(doc: &Document, var: ast::Var, mut new_name: String) -> Option<WorkspaceEdit> {
    // Ensure the new name is prefixed with $.
    if !new_name.starts_with('$') {
        new_name.insert(0, '$');
    }

    let scope_info = &doc.analysis.scope_info;
    let sym = scope_info.resolve_var(var)?;
    let edits: Vec<_> = scope_info
        .find_uses(sym, true)
        .map(|range| TextEdit::new(doc.mapper.range(range), new_name.clone()))
        .collect();

    let changes = HashMap::from([(doc.uri.clone(), edits)]);
    Some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    })
}
