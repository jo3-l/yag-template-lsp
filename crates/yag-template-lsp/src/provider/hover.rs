use tower_lsp::lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};
use yag_template_envdefs::EnvDefs;
use yag_template_syntax::ast;
use yag_template_syntax::ast::AstToken;

use crate::session::{Document, Session};

pub(crate) async fn hover(sess: &Session, params: HoverParams) -> anyhow::Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri;
    let doc = sess.document(&uri)?;

    let pos = params.text_document_position_params.position;
    let query = doc.query_at(pos);
    let hover_info = if let Some(var) = query.var() {
        hover_var(&doc, var)
    } else if query.is_in_func_call() {
        let func_ident = query.ident().unwrap();
        hover_func(&sess.envdefs, &doc, func_ident)
    } else {
        None
    };
    Ok(hover_info)
}

fn hover_var(doc: &Document, var: ast::Var) -> Option<Hover> {
    let sym = doc.analysis.scope_info.resolve_var(var.clone())?;

    let mut hover_info = format!("```\n(variable) {}\n```", var.name());
    if sym.decl_range.is_some_and(|decl| decl.contains_range(var.text_range())) {
        hover_info.push('\n');
        hover_info.push_str("Show references (Ctrl + Click) or Rename (F2)");
    } else {
        hover_info.push('\n');
        hover_info.push_str("Go to definition (Ctrl + Click) or Rename (F2)");
    }
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: hover_info,
        }),
        range: Some(doc.mapper.range(var.text_range())),
    })
}

fn hover_func(env: &EnvDefs, doc: &Document, func_ident: ast::Ident) -> Option<Hover> {
    let func = env.funcs.get(func_ident.get())?;
    let mut hover_info = format!("```ydef\n{}\n```", func.signature());
    if !func.doc.is_empty() {
        hover_info.push_str("\n\n");
        hover_info.push_str(&func.doc);
    }
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: hover_info,
        }),
        range: Some(doc.mapper.range(func_ident.text_range())),
    })
}
