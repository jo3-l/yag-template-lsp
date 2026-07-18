use yag_template_envdefs::EnvDefs;
use yag_template_syntax::ast::{ExprCall, FuncCall};

use super::LoweredExpr;
use crate::lower::Formatter;
use crate::pretty::{Doc, concat, group, soft_line, text};

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

    let row_style = if matches!(variadic.name.as_str(), "opts" | "keyvalues") {
        if !(actual_count - fixed_count).is_multiple_of(2) {
            return CallLayout::Default;
        }
        VariadicRowStyle::KeyValuePairs
    } else {
        VariadicRowStyle::Arguments
    };
    CallLayout::Variadic { fixed_count, row_style }
}

impl Formatter<'_> {
    pub(super) fn func_call(&mut self, expr: FuncCall) -> Option<LoweredExpr> {
        let name = expr.func_name()?;
        let args = expr.args().map(|arg| self.expr(arg)).collect::<Option<Vec<_>>>()?;
        let layout = classify_call(self.envdefs, name.get(), args.len());
        Some(self.call(text(name.get()), args, layout))
    }

    pub(super) fn expr_call(&mut self, expr: ExprCall) -> Option<LoweredExpr> {
        let callee = self.expr(expr.callee()?)?.into_doc();
        let args = expr.args().map(|arg| self.expr(arg)).collect::<Option<Vec<_>>>()?;
        Some(self.call(callee, args, CallLayout::Default))
    }
}

impl Formatter<'_> {
    fn call(&self, callee: Doc, args: Vec<LoweredExpr>, layout: CallLayout) -> LoweredExpr {
        let trailing_closing_group = args.last().and_then(|arg| arg.trailing_closing_group);
        if args.is_empty() {
            return LoweredExpr::new(callee);
        }
        let args = args.into_iter().map(LoweredExpr::into_doc).collect::<Vec<_>>();
        let doc = match layout {
            CallLayout::Default => group(self.callee_with_args(callee, args)),
            CallLayout::Variadic { fixed_count, row_style } => {
                let (fixed, variadic) = args.split_at(fixed_count);
                let prefix = group(self.callee_with_args(callee, fixed.iter().cloned()));
                let tail = self.variadic_tail(variadic, row_style);
                group(concat([prefix, self.indent_if_broken(tail)]))
            }
        };
        LoweredExpr {
            doc,
            trailing_closing_group,
        }
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
            VariadicRowStyle::KeyValuePairs => concat(
                args.chunks_exact(2)
                    .flat_map(|pair| [soft_line(), concat([pair[0].clone(), text(" "), pair[1].clone()])]),
            ),
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
    fn key_value_and_short_variadic_calls_fall_back_when_structure_is_unavailable() {
        let envdefs = envdefs();
        assert_eq!(classify_call(&envdefs, "mixed", 4), CallLayout::Default);
        assert_eq!(classify_call(&envdefs, "parseArgs", 2), CallLayout::Default);
        assert_eq!(classify_call(&envdefs, "parseArgs", 1), CallLayout::Default);
        assert_eq!(classify_call(&envdefs, "keys", 3), CallLayout::Default);
        assert_eq!(
            classify_call(&envdefs, "keys", 4),
            CallLayout::Variadic {
                fixed_count: 0,
                row_style: VariadicRowStyle::KeyValuePairs,
            }
        );
    }
}
