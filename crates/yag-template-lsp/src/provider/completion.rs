use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, CompletionTextEdit, TextEdit,
};
use yag_template_analysis::scope::ScopeInfo;
use yag_template_envdefs::EnvDefs;
use yag_template_syntax::ast;
use yag_template_syntax::ast::AstToken;
use yag_template_syntax::query::Query;

use crate::session::{Document, Session};

pub(crate) async fn complete(sess: &Session, params: CompletionParams) -> anyhow::Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri;
    let doc = sess.document(&uri)?;
    let pos = doc.mapper.offset(params.text_document_position.position).unwrap();

    let query = Query::at(&doc.syntax(), pos).unwrap();
    if query.is_var_access() {
        let existing_var = query.var().unwrap();
        let scope_info = &doc.analysis.scope_info;
        Ok(Some(CompletionResponse::Array(var_completion(
            &doc,
            query,
            existing_var,
            scope_info,
        ))))
    } else if query.in_func_name() {
        let existing_ident = query.ident().unwrap();
        Ok(Some(CompletionResponse::Array(func_completion(
            &sess.envdefs,
            &doc,
            existing_ident,
        ))))
    } else {
        Ok(None)
    }
}

fn var_completion(doc: &Document, query: Query, existing_var: ast::Var, scope_info: &ScopeInfo) -> Vec<CompletionItem> {
    let mut completions = Vec::new();
    for scope in scope_info.scopes_containing(query.offset) {
        completions.extend(
            scope_info[scope]
                .vars_visible_at_offset(query.offset)
                .filter(|var| var.name != existing_var.name() && var.name.starts_with(existing_var.name()))
                .map(|var| CompletionItem {
                    label: var.name.to_string(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                        new_text: var.name.to_string(),
                        range: doc.mapper.range(existing_var.syntax().text_range()).unwrap(),
                    })),
                    ..Default::default()
                }),
        )
    }
    completions
}

fn func_completion(env: &EnvDefs, doc: &Document, existing_ident: ast::Ident) -> Vec<CompletionItem> {
    env.funcs
        .values()
        .filter(|func| func.name.starts_with(existing_ident.get()))
        .map(|func| CompletionItem {
            label: func.name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: func.name.to_string(),
                range: doc.mapper.range(existing_ident.syntax().text_range()).unwrap(),
            })),
            ..Default::default()
        })
        .collect()
}
