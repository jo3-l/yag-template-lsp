use tower_lsp::lsp_types::{GotoDefinitionParams, GotoDefinitionResponse};

use crate::session::Session;

pub(crate) async fn goto_definition(
    sess: &Session,
    params: GotoDefinitionParams,
) -> anyhow::Result<Option<GotoDefinitionResponse>> {
    let uri = params.text_document_position_params.text_document.uri;
    let doc = sess.document(&uri)?;

    let pos = params.text_document_position_params.position;
    let query = doc.query_at(pos);
    let def_info = if let Some(var) = query.var() {
        doc.analysis
            .scope_info
            .resolve_var(var)
            .and_then(|sym| sym.decl_range)
            .map(|range| GotoDefinitionResponse::Scalar(doc.location_for(range)))
    } else {
        None
    };
    Ok(def_info)
}
