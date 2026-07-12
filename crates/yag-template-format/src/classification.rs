//! Resolve formatter ownership for each logical source line.

use std::collections::BTreeMap;
use std::ops::Range;

use yag_template_syntax::SyntaxNode;
use yag_template_syntax::ast::{Action, ActionList, AstNode, Expr};

use crate::line_index::LineIndex;

/// How a logical source line participates in formatter layout.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum LayoutPolicy {
    /// The formatter owns layout around actions on this line.
    Flexible,
    /// Action/text adjacency on the line stays flat; a containing block may
    /// still prefix it with indentation after an existing source newline.
    Protected,
    /// The line has no direct action. Its literal content is preserved, while
    /// leading/trailing line whitespace may become structural indentation.
    Literal,
}

/// Sparse, source-wide layout ownership derived from the typed syntax tree.
///
/// Missing entries are `Literal`; only lines containing a direct action are
/// stored. This keeps nested blocks cheap while leaving lowering free to query
/// a policy for any source line or action range.
#[derive(Debug)]
pub(super) struct LinePlan {
    line_index: LineIndex,
    policies: BTreeMap<usize, LayoutPolicy>,
}

impl LinePlan {
    /// Return the resolved policy for one physical source line.
    pub(super) fn policy_for_line(&self, line: usize) -> LayoutPolicy {
        self.policies.get(&line).copied().unwrap_or(LayoutPolicy::Literal)
    }

    /// Return the policy for the source line containing `offset`.
    #[allow(dead_code)] // Used by the later AST-to-Doc lowering pass.
    pub(super) fn policy_at_offset(&self, offset: usize) -> LayoutPolicy {
        self.policy_for_line(self.line_index.line_for(offset))
    }

    #[cfg(test)]
    fn line_count(&self) -> usize {
        self.line_index.len()
    }
}

/// Build a plan for a parsed, valid template.
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
        self.visit_sequence(root);
        LinePlan {
            line_index: self.line_index,
            policies: self.policies,
        }
    }

    /// Visit the direct actions in one root or action-list sequence.
    fn visit_sequence(&mut self, node: &SyntaxNode) {
        for action in node.children().filter_map(Action::cast) {
            self.visit_action(action);
        }
    }

    fn visit_body(&mut self, maybe_body: Option<ActionList>) {
        if let Some(body) = maybe_body {
            self.visit_sequence(body.syntax());
        }
    }

    fn visit_action(&mut self, action: Action) {
        match action {
            Action::TemplateDefinition(action) => {
                self.mark_flexible(action.clause());
                self.visit_body(action.template_body());
                self.mark_flexible(action.end_clause());
            }
            Action::TemplateBlock(action) => {
                self.mark_flexible(action.clause());
                self.visit_body(action.template_body());
                self.mark_flexible(action.end_clause());
            }
            Action::If(action) => {
                self.mark_flexible(action.clause());
                self.visit_body(action.body());
                for branch in action.else_branches() {
                    self.mark_flexible(branch.clause());
                    self.visit_body(branch.body());
                }
                self.mark_flexible(action.end_clause());
            }
            Action::With(action) => {
                self.mark_flexible(action.clause());
                self.visit_body(action.body());
                for branch in action.else_branches() {
                    self.mark_flexible(branch.clause());
                    self.visit_body(branch.body());
                }
                self.mark_flexible(action.end_clause());
            }
            Action::Range(action) => {
                self.mark_flexible(action.clause());
                self.visit_body(action.body());
                if let Some(branch) = action.else_branch() {
                    self.mark_flexible(branch.clause());
                    self.visit_body(branch.body());
                }
                self.mark_flexible(action.end_clause());
            }
            Action::While(action) => {
                self.mark_flexible(action.clause());
                self.visit_body(action.body());
                if let Some(branch) = action.else_branch() {
                    self.mark_flexible(branch.clause());
                    self.visit_body(branch.body());
                }
                self.mark_flexible(action.end_clause());
            }
            Action::TryCatch(action) => {
                self.mark_flexible(action.try_clause());
                self.visit_body(action.try_body());
                self.mark_flexible(action.catch_clause());
                self.visit_body(action.catch_body());
                self.mark_flexible(action.end_clause());
            }
            Action::ExprAction(action) => {
                let range = source_range(&action);
                let policy =
                    if action.expr().is_some_and(qualifies_display_expr) && is_single_line(&range, &self.line_index) {
                        LayoutPolicy::Protected
                    } else {
                        LayoutPolicy::Flexible
                    };
                self.mark_range(Some(action), policy);
            }
            action @ (Action::Comment(_)
            | Action::TemplateInvocation(_)
            | Action::Return(_)
            | Action::Break(_)
            | Action::Continue(_)) => self.mark_flexible(Some(action)),
        }
    }

    fn mark_flexible(&mut self, node: Option<impl AstNode>) {
        self.mark_range(node, LayoutPolicy::Flexible);
    }

    fn mark_range(&mut self, node: Option<impl AstNode>, policy: LayoutPolicy) {
        let Some(node) = node else {
            return;
        };
        let range = source_range(&node);
        if range.is_empty() {
            return;
        }
        let first_line = self.line_index.line_for(range.start);
        let last_line = self.line_index.line_for(range.end - 1);
        for line in first_line..=last_line {
            self.policies
                .entry(line)
                .and_modify(|current| {
                    // A display action protects its whole physical line,
                    // even when it shares it with a flexible action.
                    if policy == LayoutPolicy::Protected {
                        *current = LayoutPolicy::Protected;
                    }
                })
                .or_insert(policy);
        }
    }
}

fn source_range(node: &impl AstNode) -> Range<usize> {
    let range = node.text_range();
    byte_offset(range.start())..byte_offset(range.end())
}

fn byte_offset(position: impl Into<u32>) -> usize {
    position.into() as usize
}

fn qualifies_display_expr(expr: Expr) -> bool {
    match expr {
        Expr::VarAccess(_) | Expr::ContextAccess(_) | Expr::ContextFieldChain(_) => true,
        Expr::ExprFieldChain(chain) => chain.base_expr().is_some_and(qualifies_display_expr),
        Expr::Parenthesized(parenthesized) => parenthesized.inner_expr().is_some_and(qualifies_display_expr),
        _ => false,
    }
}

fn is_single_line(range: &Range<usize>, line_index: &LineIndex) -> bool {
    line_index.line_for(range.start) == line_index.line_for(range.end.saturating_sub(1))
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
        assert_eq!(policies("{{$x := 1}} {{$y := 2}}"), vec![LayoutPolicy::Flexible]);
        assert_eq!(policies("ordinary prose"), vec![LayoutPolicy::Literal]);
        assert_eq!(policies("A {{add 1 1}} C"), vec![LayoutPolicy::Flexible]);
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
    fn protected_wins_when_actions_share_a_line() {
        assert_eq!(policies("{{$value}} {{$other := 1}}"), vec![LayoutPolicy::Protected]);
    }

    #[test]
    fn multi_line_actions_are_flexible_on_every_line_they_cross() {
        assert_eq!(
            policies("{{\n  .Value\n}}\n"),
            vec![
                LayoutPolicy::Flexible,
                LayoutPolicy::Flexible,
                LayoutPolicy::Flexible,
                LayoutPolicy::Literal,
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
                LayoutPolicy::Literal,
                LayoutPolicy::Protected,
                LayoutPolicy::Flexible,
                LayoutPolicy::Flexible,
                LayoutPolicy::Flexible,
            ]
        );
    }

    #[test]
    fn every_compound_action_marks_clauses_but_not_literal_bodies() {
        for (source, expected) in [
            (
                "{{define \"name\"}}\nbody\n{{end}}",
                vec![LayoutPolicy::Flexible, LayoutPolicy::Literal, LayoutPolicy::Flexible],
            ),
            (
                "{{block \"name\" .}}\nbody\n{{end}}",
                vec![LayoutPolicy::Flexible, LayoutPolicy::Literal, LayoutPolicy::Flexible],
            ),
            (
                "{{if .Foo}}\nbody\n{{else}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Literal,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Literal,
                    LayoutPolicy::Flexible,
                ],
            ),
            (
                "{{with .Foo}}\nbody\n{{else}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Literal,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Literal,
                    LayoutPolicy::Flexible,
                ],
            ),
            (
                "{{range .Foo}}\nbody\n{{else}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Literal,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Literal,
                    LayoutPolicy::Flexible,
                ],
            ),
            (
                "{{while .Foo}}\nbody\n{{else}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Literal,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Literal,
                    LayoutPolicy::Flexible,
                ],
            ),
            (
                "{{try}}\nbody\n{{catch}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Literal,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Literal,
                    LayoutPolicy::Flexible,
                ],
            ),
        ] {
            assert_eq!(policies(source), expected, "{source}");
        }
    }

    #[test]
    fn nested_display_action_protects_a_shared_branch_clause_line() {
        assert_eq!(
            policies("{{if .Show}}{{.User}}{{else}}fallback{{end}}"),
            vec![LayoutPolicy::Protected]
        );
    }
}
