use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams, Range};
use yag_template_syntax::ast::{AstNode, SyntaxNodeExt};
use yag_template_syntax::{ast, SyntaxKind, SyntaxNode};

use crate::session::{Document, Session};

pub(crate) async fn folding_range(
    sess: &Session,
    params: FoldingRangeParams,
) -> anyhow::Result<Option<Vec<FoldingRange>>> {
    let doc = sess.document(&params.text_document.uri)?;
    let folding_ranges = doc
        .syntax()
        .descendants()
        .filter_map(|node| folding_range_for_node(&doc, node))
        .collect();
    Ok(Some(folding_ranges))
}

fn folding_range_for_node(doc: &Document, node: SyntaxNode) -> Option<FoldingRange> {
    use SyntaxKind::*;

    let range = doc.mapper.range(node.text_range());
    match node.kind() {
        TemplateDefinition => fold(range, node.to::<ast::TemplateDefinition>().define_clause()),
        TemplateBlock => fold(range, node.to::<ast::TemplateBlock>().block_clause()),
        IfConditional => fold(range, node.to::<ast::IfConditional>().if_clause()),
        ElseBranch => fold_else_branch(doc, node.to::<ast::ElseBranch>()),
        WithConditional => fold(range, node.to::<ast::WithConditional>().with_clause()),
        RangeLoop => fold(range, node.to::<ast::RangeLoop>().range_clause()),
        WhileLoop => fold(range, node.to::<ast::WhileLoop>().while_clause()),
        TryCatchAction => fold(range, node.to::<ast::TryCatchAction>().try_clause()),
        CatchClause => fold_catch_clause(doc, node.to::<ast::CatchClause>()),
        VarDecl | VarAssign => fold_var_decl_or_assign(doc, node),
        CommentAction => fold_comment(doc, node),
        _ => None,
    }
}

fn fold<N: AstNode>(r: Range, collapsed_node: Option<N>) -> Option<FoldingRange> {
    if r.start.line == r.end.line {
        return None;
    }

    Some(FoldingRange {
        start_line: r.start.line,
        start_character: Some(r.start.character),
        end_line: r.end.line,
        end_character: Some(r.end.character),
        kind: None,
        collapsed_text: collapsed_node.map(|n| n.syntax().text().to_string()),
    })
}

fn fold_else_branch(doc: &Document, else_branch: ast::ElseBranch) -> Option<FoldingRange> {
    let trimmed_range = Range {
        start: doc.mapper.position(else_branch.text_range().start()),
        end: doc.mapper.position(else_branch.action_list()?.trimmed_end_pos()),
    };
    fold(trimmed_range, else_branch.else_clause())
}

fn fold_catch_clause(doc: &Document, catch_clause: ast::CatchClause) -> Option<FoldingRange> {
    let try_catch = catch_clause.syntax().parent()?.try_to::<ast::TryCatchAction>()?;
    let range = Range {
        start: doc.mapper.position(catch_clause.text_range().start()),
        end: doc.mapper.position(try_catch.catch_action_list()?.trimmed_end_pos()),
    };
    fold(range, Some(catch_clause))
}

fn fold_var_decl_or_assign(doc: &Document, decl_or_assign: SyntaxNode) -> Option<FoldingRange> {
    let parent_action = decl_or_assign.parent()?.try_to::<ast::ExprAction>()?;
    fold::<ast::ExprAction>(doc.mapper.range(parent_action.text_range()), None)
}

fn fold_comment(doc: &Document, comment: SyntaxNode) -> Option<FoldingRange> {
    let r = doc.mapper.range(comment.text_range());
    if r.start.line == r.end.line {
        return None;
    }

    Some(FoldingRange {
        start_line: r.start.line,
        start_character: Some(r.start.character),
        end_line: r.end.line,
        end_character: Some(r.end.character),
        kind: Some(FoldingRangeKind::Comment),
        collapsed_text: None,
    })
}
