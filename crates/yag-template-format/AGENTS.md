# `yag-template-format` Guidelines

## Formatter contract

This crate formats the Go `text/template` variant parsed by
`yag-template-syntax`, including YAG `while`, `try`, `catch`, and `return`
actions. Keep formatting deterministic and idempotent.

- Never rewrite literal non-whitespace content, identifiers, pipeline order,
  parentheses, or trim markers.
- Block layout may normalize whitespace at a physical literal-text line's
  margins, but must not change whitespace between literal non-whitespace
  characters.
- Parse-invalid input must be returned byte-for-byte unchanged with parse
  diagnostics. The CLI must never write that result.
- Valid output must parse, preserve the test fingerprint, end with a
  formatter-owned terminal newline, and be byte-identical after a second
  formatting pass.

## AST and layout rules

- Format typed AST nodes; do not reconstruct actions by splitting raw source.
  `lower::lower` casts the parsed node to `Root`, and the block rules traverse
  roots and action lists through `actions_with_text()`.
- Keep ownership boundaries explicit: `block.rs` owns root/body sibling
  sequences and source-text margins, `action.rs` owns typed actions,
  `expr/` owns expression lowering, and `delimited.rs` owns action delimiters
  and their closing-row behavior. Expose narrow helpers instead of making one
  rule coordinate another rule's internals.
- Use the document model (`Text`, `Concat`, `Line`, `SoftLine`, `Group`, and
  `Indent`) for width-sensitive expression layout. `SoftLine` is a space when
  flat and a newline when broken. Build documents during lowering; only
  `pretty::render` should decide the final line breaks.
- Classify logical lines as flexible, protected-textual, or literal.
  Protected-textual lines preserve same-line text/action adjacency and force
  their actions flat. Only simple display expressions (variable/context access,
  field chains, or parentheses around them) qualify as protected-textual.
- Existing block-body newlines become structural `Line` nodes; block depth comes
  from typed block structure, not keyword scanning. Root lowering owns exactly
  one final `Line`; nested sequences return their trailing-line state to their
  caller so boundaries are not duplicated.
- Sibling separation belongs to the block sequence, not individual actions.
  Do not invent a separator at a same-line text/action boundary, and do not
  trim or reinterpret literal text in action or expression rules.
- Newlines inside an action are formatter-owned whitespace. Always lower the
  action from its typed AST; protected-textual policy preserves only same-line
  action/text adjacency, not the action's original internal layout.

## Delimiters and expressions

- Ordinary delimiters use the configured padding: `None` produces `{{foo}}`
  and `Spaces` produces `{{ foo }}`.
- Trim delimiters own their grammar-required spaces (`{{- ` and ` -}}`). Keep
  those spellings intact and apply configurable padding only to an ordinary
  delimiter on the other side. In particular, default formatting turns
  `{{- $usr := .User.String }}` into `{{- $usr := .User.String}}`.
- Newlines and indentation inside an action do not constrain formatting.
  Reflow calls, pipelines, assignments, and parentheses through the document
  model exactly as if the action had been written on one line.
- Format headers and assignments from AST nodes. Use groups, soft lines, and
  continuation nesting for calls, pipelines, and parentheses rather than
  per-case width heuristics.
- Resolve function layouts by exact syntactic callee name. Unknown names use
  normal call layout; key/value calls break one ordered pair per row. Calls
  with an odd argument count use normal call layout.

## Regression fixtures

- Every formatting behavior change needs a **new focused snapshot pair** under
  `tests/snapshots/<area>/`; do not rely only on changing a community fixture.
  Use a `.in` input and an exact `.out` output with the same basename.
- Snapshot front matter can set `max_width`, `indent`,
  `continuation_indent`, and `delimiter_padding`. Omitted fields use defaults.
- The snapshot harness verifies parseability, semantic fingerprint preservation,
  and idempotence in addition to exact output. Keep community fixtures as
  corpus coverage, and add a narrowly named action/expr/block fixture for the
  rule being fixed.

## Verification

For formatter changes, run the focused snapshot test and all crate tests:

```sh
cargo test -p yag-template-format --test format_snapshots
cargo test -p yag-template-format
```

Use `YAG_UPDATE_SNAPSHOTS=1` only when intentionally creating or updating
snapshot outputs; inspect every resulting `.out` diff.

For any Rust change, the repository also requires:

```sh
cargo +nightly fmt --all --check
cargo clippy --workspace --all-targets
```
