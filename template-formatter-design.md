# Template Formatter Design

## Scope

Implement a deterministic, idempotent formatter for the Go `text/template`
variant parsed by `yag-template-syntax`, including YAG `while`, `try`, `catch`,
and `return` actions. Deliver a reusable `yag-template-format` Rust crate and
`yag-template-fmt` CLI first; LSP integration is a separate project.

Never reflow non-whitespace literal text or rewrite literals, identifiers,
pipeline structure, parentheses, or trim behavior. Parse errors return the
original source plus diagnostics and are never written by the CLI.

## Parser and syntax requirements

Format typed AST nodes only: roots/action lists provide `actions_with_text()`,
and actions, blocks, expressions, fields, variables, literals, assignments,
comments, calls, and pipelines are represented in the syntax tree. Do not
reconstruct actions by splitting raw text. Correct `RightDelim::can_cast` in
`crates/yag-template-syntax/src/ast/tokens.rs` to accept
`SyntaxKind::TrimmedRightDelim`, with focused wrapper tests.

## API

```rust
pub struct FormatOptions {
    pub indent: Indent,
    pub continuation_indent: Indent,
    pub max_width: usize,
    pub delimiter_padding: DelimiterPadding,
    pub function_layouts: FunctionLayouts,
}
pub enum Indent { Tabs, Spaces(u8) }
pub enum DelimiterPadding { None, Spaces }
pub struct FunctionLayouts { pub by_name: BTreeMap<String, LayoutKind> }
pub enum LayoutKind { Call, KeyValuePairs { dangling_value: DanglingValuePolicy } }
pub enum DanglingValuePolicy { PreserveCallLayout, Error }
pub struct FormatResult { pub text: String, pub diagnostics: Vec<FormatDiagnostic> }
pub fn format(source: &str, options: &FormatOptions) -> FormatResult;
```

Defaults are tabs for block indentation, two spaces for continuation
indentation, width 100, delimiter padding `None`, and `dict` and `sdict` as
`KeyValuePairs` with `PreserveCallLayout`. Continuation indentation is separate
from block indentation and applies to all expression continuations.

## CLI

Provide `yag-template-fmt [OPTIONS] [FILE ...]` with `--check`, `--write`,
`--stdin-filepath`, `--width`, `--indent`, `--continuation-indent`,
`--delimiter-padding <none|spaces>`, and repeatable
`--key-value-function <NAME>`. Stdout is the default; `--check` and `--write`
are exclusive; `--write` rejects stdin. Key-value flags augment default `dict`
and `sdict` layouts.

## Algorithm

Use the standard parse, structured-document, width-aware-render model used by
Prettier and `gofmt`-style printers:

```text
source -> parser -> error validation -> region classification -> AST to Doc -> renderer
```

Implement a Wadler/Leijen-style `Doc` with `Text`, `SourceLiteral`, `Concat`,
`Line`, `SoftLine`, `Group`, and `Nest`. A bounded `fits` probe decides whether
each group is flat or broken. `SoftLine` becomes a flat space or broken newline;
`Nest` applies after newlines. `SourceLiteral` emits literal source unchanged.

Classify root/action-list sequences as structural, inline, or literal. A
structural region has a standalone action or only whitespace around it and is
formatter-owned. An inline region has literal text on the same logical line as
an action, e.g. `Hello, {{.User.Username}}!`; retain its complete sequence on
the original line and never wrap/reindent it. Literal-only regions are
byte-exact. A too-long inline action stays inline, optionally with a
non-fatal diagnostic.

Derive block depth from typed AST structure, not keywords: decrease before
`end`, `else`, `catch`; emit `else`/`catch` at parent depth; increase for their
bodies and for `if`, `with`, `range`, `while`, `try`, `define`, and `block`.

## Delimiters and expressions

Ordinary actions render as delimiter, optional padding, body, optional padding,
and delimiter. `None` emits `{{foo}}`; `Spaces` emits `{{ foo }}`. The option
does not alter trim syntax: preserve required spaces in `{{- .Value -}}` and
never generate `{{-.Value-}}`. Format headers and assignments from AST nodes.
Use `Group`, `SoftLine`, and `Nest(continuation_indent, ...)` for pipelines,
parenthesized calls, and generic calls, rather than per-case width heuristics.

## Dictionary-like functions

Resolve an exact syntactic callee name and dispatch through `FunctionLayouts`;
unknown names use `Call`. `KeyValuePairs` is a single group: flat calls join
arguments with spaces; broken calls put one ordered key/value pair per row,
nested by `continuation_indent`, with the closing parenthesis aligned to `(`.

```gotemplate
(sdict
  "a" "b"
  "c" "d"
)
```

The normal renderer chooses flat versus broken mode. Odd argument counts use
the configured policy; default behavior preserves generic call layout and emits
`OddKeyValueArgumentCount`. A configured custom callee must need no formatter
code branch.

## Preservation oracle and tests

For every valid fixture: parse original; capture a test-only owned
`TemplateFingerprint`; format; parse output without errors; compare
fingerprints; and assert a second format is byte-identical. The fingerprint
includes action/block structure, headers and assignment operators, ordered
expression trees/pipelines/calls, parentheses, exact tokens for identifiers,
literals/templates/comments, literal-text bytes, and trim markers. It excludes
only ordinary action whitespace, ordinary delimiter padding, source ranges, and
formatter-owned structural line breaks.

Do not erase parentheses or compare evaluated literal values. This is a static
semantic-shape oracle, not runtime equivalence for arbitrary user functions.
Direct oracle tests must prove intended whitespace/padding changes compare
equal, while changed pipeline order, assignment operator, parentheses, trim,
literal text, and block/else structure compare unequal.

Required fixtures cover standard and YAG actions, comments, trim, both padding
modes, width boundaries, nested blocks, prose/Markdown/code-span inline text,
`dict`/`sdict` and custom names, odd values, malformed input, and corpus
examples. Run corpus trials only on a copy of `scratch/code`, categorize diffs,
and add regression fixtures before changing templates.

## Acceptance

Run `cargo build --workspace`, `cargo test --workspace`, nightly `cargo fmt
--all`, and `cargo clippy --workspace --all-targets`. Valid output must reparse,
fingerprint equally, and be idempotent. The CLI must safely implement
stdout/check/write. Inline text must not be reflowed; delimiter options must
produce `{{foo}}` or `{{ foo }}` without invalid trim spacing; and broken
`sdict` rows must use the configured continuation indentation.
