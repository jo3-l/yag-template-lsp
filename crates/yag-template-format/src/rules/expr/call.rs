//! Function-call layout rules.
//!
//! Calls are first classified from their environment signature. An ordinary
//! call puts one argument on each broken row:
//!
//! ```text
//! {{ print
//!     "first"
//!     "second"
//! }}
//! ```
//!
//! A known variadic call keeps its fixed prefix together and puts each
//! variadic argument on a separate row:
//!
//! ```text
//! {{ joinStr "\n"
//!     .First
//!     .Second
//! }}
//! ```
//!
//! Variadic parameters named `opts` or `keyvalues` instead use one key/value
//! pair per row:
//!
//! ```text
//! {{ sdict
//!     "name" "Ada"
//!     "score" 42
//! }}
//! ```
//!
//! Finally, an ordinary call ending in a parenthesized direct call gets a
//! narrow hanging layout. The outer call through the inner callee forms one
//! group, while the inner arguments and closing parenthesis form another:
//!
//! ```text
//! {{ sendMessage nil (cembed
//!     "title" "..."
//!     "decription" "..."
//! ) }}
//! ```

use yag_template_envdefs::EnvDefs;
use yag_template_syntax::ast::{Expr, ExprCall, FuncCall};

use crate::lower::Formatter;
use crate::pretty::{Doc, concat, group, indent_if_break, line_if_break, named_group, soft_line, space, text};
use crate::rules::delimited::DelimitedInner;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum CallLayout {
    Default,
    Variadic {
        fixed_count: usize,
        row_style: VariadicRowStyle,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum VariadicRowStyle {
    Arguments,
    KeyValuePairs,
}

fn classify_call(envdefs: &EnvDefs, name: &str, actual_count: usize) -> CallLayout {
    let Some(function) = envdefs.funcs.get(name) else {
        return CallLayout::Default;
    };
    let Some(variadic) = function.params.last().filter(|param| param.is_variadic) else {
        return CallLayout::Default;
    };

    let fixed_count = function.params.len() - 1;
    if actual_count <= fixed_count {
        return CallLayout::Default;
    }

    let row_style = match variadic.name.as_str() {
        "opts" | "keyvalues" => VariadicRowStyle::KeyValuePairs,
        _ => VariadicRowStyle::Arguments,
    };
    CallLayout::Variadic { fixed_count, row_style }
}

impl Formatter<'_> {
    pub(super) fn func_call(&mut self, expr: FuncCall) -> Option<DelimitedInner> {
        let name = expr.func_name()?;
        let args = expr.args().collect::<Vec<_>>();
        let layout = classify_call(self.envdefs, name.get(), args.len());
        self.call(text(name.get()), args, layout)
    }

    pub(super) fn expr_call(&mut self, expr: ExprCall) -> Option<DelimitedInner> {
        let callee = self.expr(expr.callee()?)?.into_doc();
        self.call(callee, expr.args().collect(), CallLayout::Default)
    }

    fn call(&mut self, callee: Doc, args: Vec<Expr>, layout: CallLayout) -> Option<DelimitedInner> {
        if layout == CallLayout::Default
            && let Some(trailing_call_format) = self.try_trailing_call_format(callee.clone(), &args)
        {
            return Some(trailing_call_format);
        }

        let args = args.into_iter().map(|arg| self.expr(arg)).collect::<Option<Vec<_>>>()?;
        let trailing_closing_group = args.last().and_then(|arg| arg.trailing_closing_group);
        if args.is_empty() {
            return Some(DelimitedInner::new(callee));
        }
        let args = args.into_iter().map(DelimitedInner::into_doc).collect::<Vec<_>>();
        let doc = match layout {
            CallLayout::Default => group(self.callee_with_args(callee, args)),
            CallLayout::Variadic { fixed_count, row_style } => {
                let (fixed, variadic) = args.split_at(fixed_count);
                let prefix = group(self.callee_with_args(callee, fixed.iter().cloned()));
                let tail = self.variadic_tail(variadic, row_style);
                group(concat([prefix, self.indent_if_broken(tail)]))
            }
        };
        Some(DelimitedInner {
            doc,
            trailing_closing_group,
        })
    }

    /// Format a call ending in `(inner args...)` as two groups.
    ///
    /// For `outer "x" (inner "y" "z")`, the first group ends after
    /// `outer "x" (inner`; the inner arguments and `)` form the second group.
    fn try_trailing_call_format(&mut self, outer_callee: Doc, args: &[Expr]) -> Option<DelimitedInner> {
        // Match only an exact final `(function args...)` argument.
        let (Expr::Parenthesized(parenthesized), preceding) = args.split_last()? else {
            return None;
        };
        let Expr::FuncCall(inner_call) = parenthesized.inner_expr()? else {
            return None;
        };

        let inner_name = inner_call.func_name()?;
        let inner_args = inner_call.args().collect::<Vec<_>>();
        if inner_args.is_empty() {
            return None;
        }

        let inner_layout = classify_call(self.envdefs, inner_name.get(), inner_args.len());
        let preceding = preceding
            .iter()
            .cloned()
            .map(|arg| self.expr(arg).map(DelimitedInner::into_doc))
            .collect::<Option<Vec<_>>>()?;
        let inner_args = inner_args
            .into_iter()
            .map(|arg| self.expr(arg).map(DelimitedInner::into_doc))
            .collect::<Option<Vec<_>>>()?;
        let inner_tail = match inner_layout {
            CallLayout::Default => concat(inner_args.into_iter().flat_map(|arg| [soft_line(), arg])),
            CallLayout::Variadic { row_style, .. } => self.variadic_tail(&inner_args, row_style),
        };

        // The prefix and inner tail decide independently. If the prefix breaks,
        // the tail gains one structural indentation level.
        let prefix_id = self.new_group_id();
        let closing_id = self.new_group_id();
        let prefix = named_group(
            prefix_id,
            self.callee_with_args(
                outer_callee,
                preceding
                    .into_iter()
                    .chain([concat([text("("), text(inner_name.get())])]),
            ),
        );
        let closing = named_group(
            closing_id,
            concat([self.indent_if_broken(inner_tail), line_if_break(closing_id), text(")")]),
        );

        Some(DelimitedInner {
            doc: concat([
                prefix,
                indent_if_break(prefix_id, self.options.continuation_indent, closing),
            ]),
            trailing_closing_group: Some(closing_id),
        })
    }

    fn callee_with_args(&self, callee: Doc, args: impl IntoIterator<Item = Doc>) -> Doc {
        concat([
            callee,
            self.indent_if_broken(concat(args.into_iter().flat_map(|arg| [soft_line(), arg]))),
        ])
    }

    fn variadic_tail(&self, args: &[Doc], row_style: VariadicRowStyle) -> Doc {
        match row_style {
            VariadicRowStyle::Arguments => concat(args.iter().cloned().flat_map(|arg| [soft_line(), arg])),
            VariadicRowStyle::KeyValuePairs => {
                let pairs = args.chunks_exact(2);
                let remainder = pairs.remainder();
                concat(
                    pairs
                        .flat_map(|pair| [soft_line(), concat([pair[0].clone(), space(), pair[1].clone()])])
                        .chain(remainder.iter().cloned().flat_map(|arg| [soft_line(), arg])),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use yag_template_envdefs::EnvDefSource;

    use super::{CallLayout, VariadicRowStyle, classify_call};

    fn envdefs() -> yag_template_envdefs::EnvDefs {
        yag_template_envdefs::parse(&[EnvDefSource::new_static(
            "test.ydef",
            concat!(
                "func fixed(value)\n",
                "func values(args...)\n",
                "func parseArgs(first, description, argDefs...)\n",
                "func mixed(first, opts...)\n",
                "func keys(keyvalues...)\n",
                "func Set(opts...)\n",
            ),
        )])
        .unwrap()
    }

    #[test]
    fn call_classification_uses_variadic_signature_shape() {
        let envdefs = envdefs();
        assert_eq!(classify_call(&envdefs, "unknown", 3), CallLayout::Default);
        assert_eq!(classify_call(&envdefs, "fixed", 1), CallLayout::Default);
        assert_eq!(
            classify_call(&envdefs, "values", 3),
            CallLayout::Variadic {
                fixed_count: 0,
                row_style: VariadicRowStyle::Arguments,
            }
        );
        assert_eq!(
            classify_call(&envdefs, "parseArgs", 4),
            CallLayout::Variadic {
                fixed_count: 2,
                row_style: VariadicRowStyle::Arguments,
            }
        );
        assert_eq!(
            classify_call(&envdefs, "mixed", 5),
            CallLayout::Variadic {
                fixed_count: 1,
                row_style: VariadicRowStyle::KeyValuePairs,
            }
        );
    }

    #[test]
    fn expr_callees_do_not_use_function_signature_layouts() {
        let options = crate::FormatOptions {
            max_width: 16,
            ..crate::FormatOptions::default()
        };
        let result = crate::format(r#"{{.Set "a" "one" "b" "two"}}"#, &envdefs(), &options);
        assert!(result.diagnostics.is_empty());
        assert!(result.text.contains("\t\"a\"\n\t\"one\""), "{}", result.text);
    }

    #[test]
    fn variadic_calls_require_a_tail_but_key_value_tails_may_be_odd() {
        let envdefs = envdefs();
        assert_eq!(classify_call(&envdefs, "parseArgs", 2), CallLayout::Default);
        assert_eq!(classify_call(&envdefs, "parseArgs", 1), CallLayout::Default);
        for actual_count in [3, 4] {
            assert_eq!(
                classify_call(&envdefs, "keys", actual_count),
                CallLayout::Variadic {
                    fixed_count: 0,
                    row_style: VariadicRowStyle::KeyValuePairs,
                }
            );
        }
    }
}
