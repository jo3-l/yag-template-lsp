use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, InlayHintParams};
use yag_template_envdefs::{EnvDefs, Param};
use yag_template_syntax::ast;
use yag_template_syntax::ast::AstNode;

use crate::session::{Document, Session};

/// Only display inlay hints for functions with at least this many parameters.
const INLAY_HINT_PARAM_THRESHOLD: usize = 3;

pub(crate) async fn inlay_hint(sess: &Session, params: InlayHintParams) -> anyhow::Result<Option<Vec<InlayHint>>> {
    let doc = sess.document(&params.text_document.uri)?;
    let range = doc.mapper.text_range(params.range);
    let inlay_hints = doc
        .syntax()
        .descendants()
        .filter_map(ast::FuncCall::cast)
        .filter(|call| range.contains_range(call.syntax().text_range()))
        .flat_map(|call| inlay_hints_for_fn_call(&sess.envdefs, &doc, call))
        .flatten()
        .collect();
    Ok(Some(inlay_hints))
}

fn inlay_hints_for_fn_call<'e, 'd>(
    env: &'e EnvDefs,
    doc: &'d Document,
    call: ast::FuncCall,
) -> Option<impl Iterator<Item = InlayHint> + 'd>
where
    'e: 'd,
{
    let func = env.funcs.get(call.func_name()?.get())?;
    if func.params.len() < INLAY_HINT_PARAM_THRESHOLD {
        return None;
    }

    Some(
        call.call_args()
            .zip(func.params.iter())
            .map(|(call_expr, param)| InlayHint {
                position: doc.mapper.position(call_expr.syntax().text_range().start()),
                label: InlayHintLabel::String(param_label(param)),
                kind: Some(InlayHintKind::PARAMETER),
                text_edits: None,
                padding_right: Some(true),
                tooltip: None,
                padding_left: None,
                data: None,
            }),
    )
}

fn param_label(param: &Param) -> String {
    if param.is_variadic {
        format!("{}...", param.name)
    } else {
        format!("{}:", param.name)
    }
}
