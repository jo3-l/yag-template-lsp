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
    let resolved = doc.analysis.scope_info.resolve_var(var)?;
    let mut refs: Vec<_> = resolved
        .uses
        .iter()
        .map(|text_range| Location::new(doc_uri.clone(), doc.mapper.range(*text_range)))
        .collect();
    if context.include_declaration {
        if let Some(decl_range) = resolved.decl_range {
            refs.push(Location::new(doc_uri.clone(), doc.mapper.range(decl_range)));
        }
    }
    Some(refs)
}
