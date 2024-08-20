use tower_lsp::lsp_types::{Location, ReferenceContext, ReferenceParams, Url};
use yag_template_syntax::ast;

use crate::session::{Document, Session};

pub(crate) async fn references(sess: &Session, params: ReferenceParams) -> anyhow::Result<Option<Vec<Location>>> {
    let uri = params.text_document_position.text_document.uri;
    let doc = sess.document(&uri)?;

    let pos = doc.mapper.offset(params.text_document_position.position);
    let query = doc.query_syntax(pos)?;
    let refs = if let Some(var) = query.var() {
        find_var_references(&doc, &uri, var, &params.context)
    } else {
        None
    };
    Ok(refs)
}

fn find_var_references(
    doc: &Document,
    doc_uri: &Url,
    var: ast::Var,
    context: &ReferenceContext,
) -> Option<Vec<Location>> {
    let scope_info = &doc.analysis.scope_info;

    let sym = scope_info.resolve_var(var)?;
    let refs: Vec<_> = scope_info
        .find_uses(sym, context.include_declaration)
        .map(|range| Location::new(doc_uri.clone(), doc.mapper.range(range)))
        .collect();
    Some(refs)
}
