use tower_lsp::lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};
use yag_template_envdefs::EnvDefs;
use yag_template_syntax::ast::AstToken;
use yag_template_syntax::query::Query;
use yag_template_syntax::{ast, SyntaxNode};

use crate::session::{Document, Session};

pub(crate) async fn hover(sess: &Session, params: HoverParams) -> anyhow::Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri;
    let doc = sess.document(&uri)?;
    let pos = doc
        .mapper
        .offset(params.text_document_position_params.position)
        .unwrap();

    let root = SyntaxNode::new_root(doc.parse.root.clone());
    let query = Query::at(&root, pos).unwrap();
    if query.in_func_name() {
        let func_ident = query.ident().unwrap();
        Ok(hover_for_func(&sess.envdefs, &doc, func_ident))
    } else {
        Ok(None)
    }
}

fn hover_for_func(env: &EnvDefs, doc: &Document, func_ident: ast::Ident) -> Option<Hover> {
    let func = env.funcs.get(func_ident.get())?;
    let mut hover_info = format!("```go\n{}\n```", func.signature());
    if !func.doc.is_empty() {
        hover_info.push_str("\n\n");
        hover_info.push_str(&func.doc);
    }
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: hover_info,
        }),
        range: doc.mapper.range(func_ident.syntax().text_range()),
    })
}
