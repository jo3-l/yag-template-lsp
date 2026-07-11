//! Resolve formatter ownership for each logical source line.
//!
//! This module deliberately stores only a sparse line plan. Later AST-to-Doc
//! lowering will walk the already-parsed `Root` and `ActionList` nodes with
//! `actions_with_text()` and query this plan; it never needs to reparse source
//! or retain a second hierarchy of elements here.

use std::collections::BTreeMap;
use std::ops::Range;

use yag_template_syntax::SyntaxNode;
use yag_template_syntax::ast::{Action, AstNode, Expr};

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
    Verbatim,
}

/// Sparse, source-wide layout ownership derived from the typed syntax tree.
///
/// Missing entries are `Verbatim`; only lines containing a direct action are
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
        self.policies.get(&line).copied().unwrap_or(LayoutPolicy::Verbatim)
    }

    /// Return the policy for the source line containing `offset`.
    #[allow(dead_code)] // Used by the later AST-to-Doc lowering pass.
    pub(super) fn policy_at_offset(&self, offset: usize) -> LayoutPolicy {
        self.policy_for_line(self.line_index.line_for(offset))
    }

    /// Whether any source line covered by `range` is protected.
    ///
    /// A cross-line action that cannot be emitted as independently pinned AST
    /// pieces must fall back to its original source when this is true.
    #[allow(dead_code)] // Used by the later AST-to-Doc lowering pass.
    pub(super) fn range_contains_protected_line(&self, range: Range<usize>) -> bool {
        if range.is_empty() {
            return false;
        }
        let first_line = self.line_index.line_for(range.start);
        let last_line = self.line_index.line_for(range.end - 1);
        self.policies
            .range(first_line..=last_line)
            .any(|(_, policy)| *policy == LayoutPolicy::Protected)
    }

    pub(super) fn protected_textual_line_mask(&self) -> Vec<bool> {
        let mut protected = vec![false; self.line_index.len()];
        for (&line, &policy) in &self.policies {
            if policy == LayoutPolicy::Protected {
                protected[line] = true;
            }
        }
        protected
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

    fn visit_action(&mut self, action: Action) {
        match action {
            Action::TemplateDefinition(action) => {
                self.mark_range(action.clause(), LayoutPolicy::Flexible);
                if let Some(body) = action.template_body() {
                    self.visit_sequence(body.syntax());
                }
                self.mark_range(action.end_clause(), LayoutPolicy::Flexible);
            }
            Action::TemplateBlock(action) => {
                self.mark_range(action.clause(), LayoutPolicy::Flexible);
                if let Some(body) = action.template_body() {
                    self.visit_sequence(body.syntax());
                }
                self.mark_range(action.end_clause(), LayoutPolicy::Flexible);
            }
            Action::If(action) => {
                self.mark_range(action.clause(), LayoutPolicy::Flexible);
                if let Some(body) = action.body() {
                    self.visit_sequence(body.syntax());
                }
                for branch in action.else_branches() {
                    self.mark_range(branch.clause(), LayoutPolicy::Flexible);
                    if let Some(body) = branch.body() {
                        self.visit_sequence(body.syntax());
                    }
                }
                self.mark_range(action.end_clause(), LayoutPolicy::Flexible);
            }
            Action::With(action) => {
                self.mark_range(action.clause(), LayoutPolicy::Flexible);
                if let Some(body) = action.body() {
                    self.visit_sequence(body.syntax());
                }
                for branch in action.else_branches() {
                    self.mark_range(branch.clause(), LayoutPolicy::Flexible);
                    if let Some(body) = branch.body() {
                        self.visit_sequence(body.syntax());
                    }
                }
                self.mark_range(action.end_clause(), LayoutPolicy::Flexible);
            }
            Action::Range(action) => {
                self.mark_range(action.clause(), LayoutPolicy::Flexible);
                if let Some(body) = action.body() {
                    self.visit_sequence(body.syntax());
                }
                if let Some(branch) = action.else_branch() {
                    self.mark_range(branch.clause(), LayoutPolicy::Flexible);
                    if let Some(body) = branch.body() {
                        self.visit_sequence(body.syntax());
                    }
                }
                self.mark_range(action.end_clause(), LayoutPolicy::Flexible);
            }
            Action::While(action) => {
                self.mark_range(action.clause(), LayoutPolicy::Flexible);
                if let Some(body) = action.body() {
                    self.visit_sequence(body.syntax());
                }
                if let Some(branch) = action.else_branch() {
                    self.mark_range(branch.clause(), LayoutPolicy::Flexible);
                    if let Some(body) = branch.body() {
                        self.visit_sequence(body.syntax());
                    }
                }
                self.mark_range(action.end_clause(), LayoutPolicy::Flexible);
            }
            Action::TryCatch(action) => {
                self.mark_range(action.try_clause(), LayoutPolicy::Flexible);
                if let Some(body) = action.try_body() {
                    self.visit_sequence(body.syntax());
                }
                self.mark_range(action.catch_clause(), LayoutPolicy::Flexible);
                if let Some(body) = action.catch_body() {
                    self.visit_sequence(body.syntax());
                }
                self.mark_range(action.end_clause(), LayoutPolicy::Flexible);
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
            | Action::Continue(_)) => self.mark_range(Some(action), LayoutPolicy::Flexible),
        }
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
        assert_eq!(policies("ordinary prose"), vec![LayoutPolicy::Verbatim]);
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
                LayoutPolicy::Verbatim,
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
                LayoutPolicy::Verbatim,
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
                vec![LayoutPolicy::Flexible, LayoutPolicy::Verbatim, LayoutPolicy::Flexible],
            ),
            (
                "{{block \"name\" .}}\nbody\n{{end}}",
                vec![LayoutPolicy::Flexible, LayoutPolicy::Verbatim, LayoutPolicy::Flexible],
            ),
            (
                "{{if .Foo}}\nbody\n{{else}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Verbatim,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Verbatim,
                    LayoutPolicy::Flexible,
                ],
            ),
            (
                "{{with .Foo}}\nbody\n{{else}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Verbatim,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Verbatim,
                    LayoutPolicy::Flexible,
                ],
            ),
            (
                "{{range .Foo}}\nbody\n{{else}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Verbatim,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Verbatim,
                    LayoutPolicy::Flexible,
                ],
            ),
            (
                "{{while .Foo}}\nbody\n{{else}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Verbatim,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Verbatim,
                    LayoutPolicy::Flexible,
                ],
            ),
            (
                "{{try}}\nbody\n{{catch}}\nother\n{{end}}",
                vec![
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Verbatim,
                    LayoutPolicy::Flexible,
                    LayoutPolicy::Verbatim,
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

    #[test]
    fn cross_line_actions_can_detect_a_protected_boundary_for_verbatim_fallback() {
        let source = "{{add\n  1\n}}{{.Value}}";
        assert!(plan(source).range_contains_protected_line(0.."{{add\n  1\n}}".len()));
    }

    #[test]
    fn diagnostic_mask_is_derived_from_the_same_plan() {
        let plan = plan("prose\nA {{.Value}} C\n{{$x := 1}}");
        assert_eq!(plan.protected_textual_line_mask(), vec![false, true, false]);
    }
}
