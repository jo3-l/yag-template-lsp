//! Resolve formatter ownership for each logical source line.

use std::collections::BTreeMap;
use std::ops::Range;

use yag_template_syntax::ast::{Action, AstNode, Expr};
use yag_template_syntax::{SyntaxKind, SyntaxNode};

use crate::line_index::LineIndex;

/// How a logical source line participates in formatter layout.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum LayoutPolicy {
    /// The formatter may freely rewrite whitespace between actions and text
    /// on this line.
    Flexible,
    /// The formatter must not rewrite whitespace between actions and text
    /// on this line. It may still reformat internally within actions.
    Protected,
}

/// Sparse, source-wide reflow freedom derived from the typed syntax tree.
/// Missing entries are flexible.
#[derive(Debug)]
pub(super) struct LinePlan {
    line_index: LineIndex,
    policies: BTreeMap<usize, LayoutPolicy>,
}

impl LinePlan {
    /// Return the resolved policy for one physical source line.
    pub(super) fn policy_for_line(&self, line: usize) -> LayoutPolicy {
        self.policies.get(&line).copied().unwrap_or(LayoutPolicy::Flexible)
    }

    /// Return the policy for the source line containing `offset`.
    pub(super) fn policy_at_offset(&self, offset: usize) -> LayoutPolicy {
        self.policy_for_line(self.line_index.line_for(offset))
    }

    #[cfg(test)]
    fn line_count(&self) -> usize {
        self.line_index.len()
    }
}

pub(super) fn classify(root: &SyntaxNode, source: &str) -> LinePlan {
    LineClassifier::new(source).classify(root)
}

struct LineClassifier {
    line_index: LineIndex,
    policies: BTreeMap<usize, LayoutPolicy>,
}

impl LineClassifier {
    fn new(source: &str) -> Self {
        Self {
            line_index: LineIndex::new(source),
            policies: BTreeMap::new(),
        }
    }

    fn classify(mut self, root: &SyntaxNode) -> LinePlan {
        self.protect_text_lines(root);
        self.protect_display_lines(root);
        LinePlan {
            line_index: self.line_index,
            policies: self.policies,
        }
    }

    /// Literal non-whitespace protects its physical source line.
    fn protect_text_lines(&mut self, root: &SyntaxNode) {
        for token in root
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
            .filter(|token| token.kind() == SyntaxKind::Text)
        {
            let start = byte_offset(token.text_range().start());
            for (offset, c) in token.text().char_indices() {
                if !c.is_whitespace() {
                    self.protect_line(self.line_index.line_for(start + offset));
                }
            }
        }
    }

    fn protect_display_lines(&mut self, root: &SyntaxNode) {
        for action in root.descendants().filter_map(Action::cast) {
            let Action::ExprAction(action) = action else {
                continue;
            };
            if action.expr().is_some_and(is_output_expression) {
                self.protect_range(source_range(&action));
            }
        }
    }

    fn protect_range(&mut self, range: Range<usize>) {
        if range.is_empty() {
            return;
        }
        let first_line = self.line_index.line_for(range.start);
        let last_line = self.line_index.line_for(range.end - 1);
        for line in first_line..=last_line {
            self.protect_line(line);
        }
    }

    fn protect_line(&mut self, line: usize) {
        self.policies.insert(line, LayoutPolicy::Protected);
    }
}

fn source_range(node: &impl AstNode) -> Range<usize> {
    let range = node.text_range();
    byte_offset(range.start())..byte_offset(range.end())
}

fn byte_offset(position: impl Into<u32>) -> usize {
    position.into() as usize
}

/// Whether `expr` has a display-like shape that protects its physical line.
/// Calls, pipelines, and assignments deliberately do not qualify.
fn is_output_expression(expr: Expr) -> bool {
    match expr {
        Expr::VarAccess(_) | Expr::ContextAccess(_) | Expr::ContextFieldChain(_) | Expr::ExprFieldChain(_) => true,
        Expr::Parenthesized(parenthesized) => parenthesized.inner_expr().is_some_and(is_output_expression),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use yag_template_syntax::SyntaxNode;

    use super::{LayoutPolicy, LinePlan, classify};

    fn plan(source: &str) -> LinePlan {
        let parsed = yag_template_syntax::parser::parse(source);
        assert!(parsed.errors.is_empty(), "source did not parse: {:?}", parsed.errors);
        classify(&SyntaxNode::new_root(parsed.root), source)
    }

    fn policies(source: &str) -> Vec<LayoutPolicy> {
        let plan = plan(source);
        (0..plan.line_count()).map(|line| plan.policy_for_line(line)).collect()
    }

    #[test]
    fn motivating_examples_resolve_to_the_expected_line_policies() {
        assert_eq!(policies("A {{$b}} C"), vec![LayoutPolicy::Protected]);
        assert_eq!(policies("{{$b}} {{$c}}"), vec![LayoutPolicy::Protected]);
        assert_eq!(policies("{{$x := 1}} {{$y := 2}}"), vec![LayoutPolicy::Flexible]);
        assert_eq!(policies("ordinary prose"), vec![LayoutPolicy::Protected]);
        assert_eq!(policies("A {{add 1 1}} C"), vec![LayoutPolicy::Protected]);
    }

    #[test]
    fn typed_display_shapes_are_protected_but_calls_pipelines_and_assignments_are_not() {
        for source in [
            "{{$value}}",
            "{{.}}",
            "{{.User.Name}}",
            "{{$value.Name}}",
            "{{(.User.Name)}}",
        ] {
            assert_eq!(policies(source), vec![LayoutPolicy::Protected], "{source}");
        }
        for source in ["{{$value := 1}}", "{{add 1 1}}", "{{$value | printf \"%v\"}}"] {
            assert_eq!(policies(source), vec![LayoutPolicy::Flexible], "{source}");
        }
    }

    #[test]
    fn protected_display_wins_when_actions_share_a_line() {
        assert_eq!(policies("{{$value}} {{$other := 1}}"), vec![LayoutPolicy::Protected]);
    }

    #[test]
    fn display_actions_protect_every_line_they_cross() {
        assert_eq!(
            policies("{{\n  .Value\n}}\n"),
            vec![
                LayoutPolicy::Protected,
                LayoutPolicy::Protected,
                LayoutPolicy::Protected,
                LayoutPolicy::Flexible,
            ]
        );
    }

    #[test]
    fn compound_bodies_classify_independently_of_their_boundaries() {
        let source = "{{if .Show}}\nordinary prose\nA {{.User.Name}} C\n{{else}}\n{{$x := 1}} {{$y := 2}}\n{{end}}";
        assert_eq!(
            policies(source),
            vec![
                LayoutPolicy::Flexible,
                LayoutPolicy::Protected,
                LayoutPolicy::Protected,
                LayoutPolicy::Flexible,
                LayoutPolicy::Flexible,
                LayoutPolicy::Flexible,
            ]
        );
    }

    #[test]
    fn compound_actions_leave_non_text_lines_flexible_and_protect_literal_bodies() {
        for (source, expected) in [
            (
                "{{define \"name\"}}\nbody\n{{end}}",
                vec![LayoutPolicy::Flexible, LayoutPolicy::Protected, LayoutPolicy::Flexible],
            ),
            (
                "{{block \"name\" .}}\nbody\n{{end}}",
                vec![LayoutPolicy::Flexible, LayoutPolicy::Protected, LayoutPolicy::Flexible],
            ),
            (
                "{{if .Foo}}\nbody\n{{else}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Protected,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Protected,
                    LayoutPolicy::Flexible,
                ],
            ),
            (
                "{{with .Foo}}\nbody\n{{else}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Protected,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Protected,
                    LayoutPolicy::Flexible,
                ],
            ),
            (
                "{{range .Foo}}\nbody\n{{else}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Protected,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Protected,
                    LayoutPolicy::Flexible,
                ],
            ),
            (
                "{{while .Foo}}\nbody\n{{else}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Protected,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Protected,
                    LayoutPolicy::Flexible,
                ],
            ),
            (
                "{{try}}\nbody\n{{catch}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Protected,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Protected,
                    LayoutPolicy::Flexible,
                ],
            ),
        ] {
            assert_eq!(policies(source), expected, "{source}");
        }
    }

    #[test]
    fn display_actions_protect_their_physical_line() {
        assert_eq!(
            policies("{{if .Show}}{{.User}}{{else}}fallback{{end}}"),
            vec![LayoutPolicy::Protected]
        );
    }
}
