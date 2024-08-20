use tower_lsp::lsp_types::{Location, ReferenceContext, ReferenceParams};
use yag_template_syntax::ast::{self, AstNode};

use crate::session::{Document, Session};

pub(crate) async fn references(sess: &Session, params: ReferenceParams) -> anyhow::Result<Option<Vec<Location>>> {
    let uri = params.text_document_position.text_document.uri;
    let doc = sess.document(&uri)?;

    let pos = params.text_document_position.position;
    let query = doc.query_at(pos)?;
    let refs = if let Some(var) = query.var() {
        find_var_references(&doc, var, &params.context)
    } else if query.in_func_name() {
        let func_ident = query.ident().unwrap();
        find_func_references(&doc, func_ident.get())
    } else {
        None
    };
    Ok(refs)
}

fn find_var_references(doc: &Document, var: ast::Var, context: &ReferenceContext) -> Option<Vec<Location>> {
    let scope_info = &doc.analysis.scope_info;

    let sym = scope_info.resolve_var(var)?;
    let refs: Vec<_> = scope_info
        .find_uses(sym, context.include_declaration)
        .map(|range| doc.location_for(range))
        .collect();
    Some(refs)
}

fn find_func_references(doc: &Document, func_name: &str) -> Option<Vec<Location>> {
    let refs: Vec<_> = doc
        .syntax()
        .descendants()
        .filter_map(ast::FuncCall::cast)
        .filter(|call| call.func_name().is_some_and(|call_name| call_name.get() == func_name))
        .map(|call| doc.location_for(call.syntax().text_range()))
        .collect();
    Some(refs)
}
