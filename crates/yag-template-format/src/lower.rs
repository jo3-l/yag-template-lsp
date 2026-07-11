//! AST-to-document lowering for the formatter's conservative early stages.
//!
//! This pass deliberately changes only ordinary delimiter padding. Expression
//! and block layout are added in later milestones; trim, comments, and
//! multi-line actions consequently remain source-verbatim here.

use std::ops::Range;

use yag_template_syntax::ast::{
    Action, ActionList, ActionOrText, AstNode, CommentAction, Root, TemplateBlock, TemplateDefinition, TryCatchAction,
};
use yag_template_syntax::{SyntaxKind, SyntaxNode};

use crate::doc::Doc;
use crate::region::LinePlan;
use crate::{DelimiterPadding, FormatOptions};

pub(super) fn lower(root: &SyntaxNode, source: &str, options: &FormatOptions, plan: &LinePlan) -> Doc {
    let Some(root) = Root::cast(root.clone()) else {
        return Doc::verbatim(source);
    };
    Lowerer { source, options, plan }.root(root)
}

struct Lowerer<'a> {
    source: &'a str,
    options: &'a FormatOptions,
    plan: &'a LinePlan,
}

impl Lowerer<'_> {
    fn root(&self, root: Root) -> Doc {
        self.sequence(root.actions_with_text())
    }

    fn action_list(&self, action_list: ActionList) -> Doc {
        self.sequence(action_list.actions_with_text())
    }

    fn sequence(&self, elements: impl Iterator<Item = ActionOrText>) -> Doc {
        Doc::concat(elements.map(|element| match element {
            ActionOrText::Action(action) => self.action(action),
            ActionOrText::Text(text) => Doc::verbatim(text.get()),
        }))
    }

    fn action(&self, action: Action) -> Doc {
        match action {
            Action::TemplateDefinition(action) => self.template_definition(action),
            Action::TemplateBlock(action) => self.template_block(action),
            Action::If(action) => self.if_action(action),
            Action::With(action) => self.with_action(action),
            Action::Range(action) => self.range_action(action),
            Action::While(action) => self.while_action(action),
            Action::TryCatch(action) => self.try_catch_action(action),
            Action::Comment(action) => self.comment(action),
            action @ (Action::TemplateInvocation(_)
            | Action::Return(_)
            | Action::Break(_)
            | Action::Continue(_)
            | Action::ExprAction(_)) => self.delimited(action),
        }
    }

    fn comment(&self, action: CommentAction) -> Doc {
        // Go template comments require the comment marker to remain adjacent
        // to the opening delimiter. Keep comments exact under both padding
        // modes rather than accidentally producing a non-portable spelling.
        self.verbatim(&action)
    }

    fn template_definition(&self, action: TemplateDefinition) -> Doc {
        self.compound(
            &action,
            [
                action.clause().map(|clause| self.delimited(clause)),
                action.template_body().map(|body| self.action_list(body)),
                action.end_clause().map(|clause| self.delimited(clause)),
            ],
        )
    }

    fn template_block(&self, action: TemplateBlock) -> Doc {
        self.compound(
            &action,
            [
                action.clause().map(|clause| self.delimited(clause)),
                action.template_body().map(|body| self.action_list(body)),
                action.end_clause().map(|clause| self.delimited(clause)),
            ],
        )
    }

    fn if_action(&self, action: yag_template_syntax::ast::IfAction) -> Doc {
        let mut parts = vec![
            action.clause().map(|clause| self.delimited(clause)),
            action.body().map(|body| self.action_list(body)),
        ];
        for branch in action.else_branches() {
            parts.push(branch.clause().map(|clause| self.delimited(clause)));
            parts.push(branch.body().map(|body| self.action_list(body)));
        }
        parts.push(action.end_clause().map(|clause| self.delimited(clause)));
        self.compound(&action, parts)
    }

    fn with_action(&self, action: yag_template_syntax::ast::WithAction) -> Doc {
        let mut parts = vec![
            action.clause().map(|clause| self.delimited(clause)),
            action.body().map(|body| self.action_list(body)),
        ];
        for branch in action.else_branches() {
            parts.push(branch.clause().map(|clause| self.delimited(clause)));
            parts.push(branch.body().map(|body| self.action_list(body)));
        }
        parts.push(action.end_clause().map(|clause| self.delimited(clause)));
        self.compound(&action, parts)
    }

    fn range_action(&self, action: yag_template_syntax::ast::RangeLoop) -> Doc {
        self.compound(
            &action,
            [
                action.clause().map(|clause| self.delimited(clause)),
                action.body().map(|body| self.action_list(body)),
                action
                    .else_branch()
                    .and_then(|branch| branch.clause().map(|clause| self.delimited(clause))),
                action
                    .else_branch()
                    .and_then(|branch| branch.body().map(|body| self.action_list(body))),
                action.end_clause().map(|clause| self.delimited(clause)),
            ],
        )
    }

    fn while_action(&self, action: yag_template_syntax::ast::WhileLoop) -> Doc {
        self.compound(
            &action,
            [
                action.clause().map(|clause| self.delimited(clause)),
                action.body().map(|body| self.action_list(body)),
                action
                    .else_branch()
                    .and_then(|branch| branch.clause().map(|clause| self.delimited(clause))),
                action
                    .else_branch()
                    .and_then(|branch| branch.body().map(|body| self.action_list(body))),
                action.end_clause().map(|clause| self.delimited(clause)),
            ],
        )
    }

    fn try_catch_action(&self, action: TryCatchAction) -> Doc {
        self.compound(
            &action,
            [
                action.try_clause().map(|clause| self.delimited(clause)),
                action.try_body().map(|body| self.action_list(body)),
                action.catch_clause().map(|clause| self.delimited(clause)),
                action.catch_body().map(|body| self.action_list(body)),
                action.end_clause().map(|clause| self.delimited(clause)),
            ],
        )
    }

    fn compound<N: AstNode>(&self, action: &N, parts: impl IntoIterator<Item = Option<Doc>>) -> Doc {
        let parts = parts.into_iter().collect::<Option<Vec<_>>>();
        parts.map_or_else(|| self.verbatim(action), Doc::concat)
    }

    fn delimited<N: AstNode>(&self, node: N) -> Doc {
        let range = source_range(&node);
        let source = &self.source[range.clone()];
        if self.plan.range_contains_protected_line(range) && source.contains('\n') {
            return Doc::verbatim(source);
        }

        let Some(left) = node.syntax().first_token() else {
            return Doc::verbatim(source);
        };
        let Some(right) = node.syntax().last_token() else {
            return Doc::verbatim(source);
        };
        if !matches!(left.kind(), SyntaxKind::LeftDelim | SyntaxKind::TrimmedLeftDelim)
            || !matches!(right.kind(), SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim)
        {
            return Doc::verbatim(source);
        }

        // Preserve trim spellings exactly. In particular, `{{- .Value -}}`
        // has whitespace required by the trim-marker grammar.
        if matches!(left.kind(), SyntaxKind::TrimmedLeftDelim)
            || matches!(right.kind(), SyntaxKind::TrimmedRightDelim)
            || source.contains('\n')
        {
            return Doc::verbatim(source);
        }

        let left_end = byte_offset(left.text_range().end());
        let right_start = byte_offset(right.text_range().start());
        let body = self.source[left_end..right_start].trim();
        let padding = match self.options.delimiter_padding {
            DelimiterPadding::None => "",
            DelimiterPadding::Spaces => " ",
        };
        Doc::concat([
            Doc::text(left.text()),
            Doc::text(padding),
            Doc::verbatim(body),
            Doc::text(padding),
            Doc::text(right.text()),
        ])
    }

    fn verbatim<N: AstNode>(&self, node: &N) -> Doc {
        Doc::verbatim(&self.source[source_range(node)])
    }
}

fn source_range(node: &impl AstNode) -> Range<usize> {
    let range = node.text_range();
    byte_offset(range.start())..byte_offset(range.end())
}

fn byte_offset(position: impl Into<u32>) -> usize {
    position.into() as usize
}
