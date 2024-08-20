use tower_lsp::lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location};

use crate::session::Session;

pub(crate) async fn goto_definition(
    sess: &Session,
    params: GotoDefinitionParams,
) -> anyhow::Result<Option<GotoDefinitionResponse>> {
    let uri = params.text_document_position_params.text_document.uri;
    let doc = sess.document(&uri)?;

    let pos = doc.mapper.offset(params.text_document_position_params.position);
    let query = doc.query_syntax(pos)?;
    let def_info = if let Some(var) = query.var() {
        doc.analysis
            .scope_info
            .resolve_var(var)
            .and_then(|resolved| resolved.decl_range)
            .map(|decl_range| GotoDefinitionResponse::Scalar(Location::new(uri, doc.mapper.range(decl_range))))
    } else {
        None
    };
    Ok(def_info)
}
