use yag_template_envdefs::EnvDefs;
use yag_template_syntax::ast::{ExprCall, FuncCall};

use super::ExprDoc;
use crate::lower::Formatter;
use crate::pretty::{Doc, concat, group, soft_line, text};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum CallLayout {
    Default,
    Variadic { fixed_count: usize, rows: VariadicRows },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum VariadicRows {
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

    let rows = if matches!(variadic.name.as_str(), "opts" | "keyvalues") {
        if !(actual_count - fixed_count).is_multiple_of(2) {
            return CallLayout::Default;
        }
        VariadicRows::KeyValuePairs
    } else {
        VariadicRows::Arguments
    };
    CallLayout::Variadic { fixed_count, rows }
}

impl Formatter<'_> {
    pub(super) fn func_call(&mut self, expr: FuncCall) -> Option<ExprDoc> {
        let name = expr.func_name()?;
        let args = expr.args().map(|arg| self.expr_doc(arg)).collect::<Option<Vec<_>>>()?;
        Some(self.function_call(name.get(), args))
    }

    pub(super) fn expr_call(&mut self, expr: ExprCall) -> Option<ExprDoc> {
        let callee = self.expr_doc(expr.callee()?)?.into_doc();
        let args = expr.args().map(|arg| self.expr_doc(arg)).collect::<Option<Vec<_>>>()?;
        Some(self.call(callee, args))
    }
}

impl Formatter<'_> {
    fn function_call(&mut self, name: &str, args: Vec<ExprDoc>) -> ExprDoc {
        // Slice 5 will construct all signature-guided layouts. For now, retain
        // the existing output gate while deriving that decision from the full classifier.
        match classify_call(self.envdefs, name, args.len()) {
            CallLayout::Variadic {
                fixed_count: 0,
                rows: VariadicRows::KeyValuePairs,
            } => self.key_value_call(text(name), args),
            CallLayout::Default | CallLayout::Variadic { .. } => self.call(text(name), args),
        }
    }

    fn call(&self, callee: Doc, args: impl IntoIterator<Item = ExprDoc>) -> ExprDoc {
        let args = args.into_iter().collect::<Vec<_>>();
        if args.is_empty() {
            return ExprDoc::new(callee);
        }
        let trailing_closing_group = args.last().and_then(|arg| arg.trailing_closing_group);
        ExprDoc {
            doc: group(concat([
                callee,
                self.indent_if_broken(concat(args.into_iter().flat_map(|arg| [soft_line(), arg.into_doc()]))),
            ])),
            trailing_closing_group,
        }
    }

    fn key_value_call(&self, callee: Doc, args: Vec<ExprDoc>) -> ExprDoc {
        let trailing_closing_group = args.last().and_then(|arg| arg.trailing_closing_group);
        let args = args.into_iter().map(ExprDoc::into_doc).collect::<Vec<_>>();
        let rows = args
            .chunks_exact(2)
            .flat_map(|pair| [soft_line(), pair[0].clone(), text(" "), pair[1].clone()]);
        ExprDoc {
            doc: group(concat([callee, self.indent_if_broken(concat(rows))])),
            trailing_closing_group,
        }
    }
}

#[cfg(test)]
mod tests {
    use yag_template_envdefs::EnvDefSource;

    use super::{CallLayout, VariadicRows, classify_call};

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
                rows: VariadicRows::Arguments,
            }
        );
        assert_eq!(
            classify_call(&envdefs, "parseArgs", 4),
            CallLayout::Variadic {
                fixed_count: 2,
                rows: VariadicRows::Arguments,
            }
        );
        assert_eq!(
            classify_call(&envdefs, "mixed", 5),
            CallLayout::Variadic {
                fixed_count: 1,
                rows: VariadicRows::KeyValuePairs,
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
                rows: VariadicRows::KeyValuePairs,
            }
        );
    }
}
