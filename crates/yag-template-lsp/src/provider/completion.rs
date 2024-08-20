use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, CompletionTextEdit, TextEdit,
};
use yag_template_analysis::scope::VarSymbol;
use yag_template_envdefs::EnvDefs;
use yag_template_syntax::ast;
use yag_template_syntax::ast::AstToken;
use yag_template_syntax::query::Query;

use crate::session::{Document, Session};

pub(crate) async fn complete(sess: &Session, params: CompletionParams) -> anyhow::Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri;
    let doc = sess.document(&uri)?;

    let pos = params.text_document_position.position;
    let query = doc.query_at(pos)?;
    let completions = if query.in_var_access() {
        let existing_var = query.var().unwrap();
        let completions = complete_var(&doc, query, existing_var);
        Some(CompletionResponse::Array(completions))
    } else if query.in_func_name() {
        let existing_ident = query.ident().unwrap();
        let completions = complete_func(&sess.envdefs, &doc, existing_ident);
        Some(CompletionResponse::Array(completions))
    } else {
        None
    };
    Ok(completions)
}

fn complete_var(doc: &Document, query: Query, existing_var: ast::Var) -> Vec<CompletionItem> {
    let generate_completion = |var: &VarSymbol| CompletionItem {
        label: var.name.to_string(),
        kind: Some(CompletionItemKind::VARIABLE),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            new_text: var.name.to_string(),
            range: doc.mapper.range(existing_var.syntax().text_range()),
        })),
        ..Default::default()
    };

    doc.analysis
        .scope_info
        .scopes_containing(query.offset)
        .flat_map(|scope| {
            scope
                .vars_visible_at_offset(query.offset)
                .filter(|var| var.name != existing_var.name() && var.name.starts_with(existing_var.name()))
        })
        .map(generate_completion)
        .collect()
}

fn complete_func(env: &EnvDefs, doc: &Document, existing_ident: ast::Ident) -> Vec<CompletionItem> {
    env.funcs
        .values()
        .filter(|func| func.name.starts_with(existing_ident.get()))
        .map(|func| CompletionItem {
            label: func.name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: func.name.to_string(),
                range: doc.mapper.range(existing_ident.syntax().text_range()),
            })),
            ..Default::default()
        })
        .collect()
}
