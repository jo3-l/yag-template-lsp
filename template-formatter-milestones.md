# Template Formatter Delivery Milestones

This is the small-commit execution roadmap for [the formatter design](template-formatter-design.md). Do not start a milestone until its verifier passes and its required human review is complete.

## Working rules

- One concern and its tests per commit; no unrelated churn.
- Add a regression fixture with every behavior change.
- Each valid fixture proves expected output, output parsing, equal `TemplateFingerprint`, and idempotence as soon as that support exists.
- Use a copy of `scratch/code` for corpus trials. Never format repository templates in place during development.

## 0. Baseline and fixture inventory

Collect representative `scratch/code` templates: inline prose, blocks, pipelines, comments, trim markers, compressed source, and `dict`/`sdict`; copy minimal examples into fixture inputs and record their source paths.

**Verifier:** `cargo test --workspace` passes and selected valid examples parse.

**Human gate:** Confirm output-sensitive templates are represented.

**Commit:** `format: add formatter fixture inventory`

## 1. Syntax prerequisite

Correct `RightDelim::can_cast` to recognize `TrimmedRightDelim`; add normal/trimmed left/right token-wrapper tests.

**Verifier:** Focused tests and `cargo test -p yag-template-syntax`.

**Human gate:** None required; contained bug fix.

**Commit:** `syntax: expose trimmed right delimiters correctly`

## 2. Formatter crate and public API

Create `yag-template-format`, option/diagnostic types, defaults, parser integration, and a conservative `format` stub that leaves valid input unchanged and returns malformed input unchanged with diagnostics.

**Verifier:** `cargo test -p yag-template-format` and `cargo build --workspace`.

**Human gate:** Approve type names, defaults, diagnostics, and crate/binary placement.

**Commit:** `format: add formatter crate and options`

## 3. Semantic fingerprint oracle

Add a test-only AST `TemplateFingerprint` plus shared fixture assertion: parse and fingerprint input, format, parse and fingerprint output, compare, and assert idempotence. Include action/block structure, ordered expressions/pipelines, parentheses, exact literals/comments/text, assignments, and trim; ignore only action whitespace, ordinary delimiter padding, ranges, and formatter-owned structural breaks.

**Verifier:** Tests show whitespace/padding compare equal, while pipeline order, assignment, parentheses, trim, literal text, and branches compare unequal; run `cargo test -p yag-template-format fingerprint`.

**Human gate:** Approve fingerprint fields and exclusions as the preservation contract.

**Commit:** `format: add semantic fingerprint test oracle`

## 4. Doc algebra and renderer

Implement internal `Doc`, flattening, bounded `fits`, nesting, hard/soft lines, and exact source-literal rendering with no AST integration.

**Verifier:** Table-driven width-boundary tests for groups, nesting, hard/soft lines, and literals; run `cargo test -p yag-template-format doc`.

**Human gate:** Review `fits` semantics before AST lowering.

**Commit:** `format: add width-aware document renderer`

## 5. Region classification and no-reflow protection

Classify root/action-list sequences as structural, inline, or literal. Emit literal regions byte-exact; retain complete inline sequences on their original line; optionally diagnose protected over-width lines.

**Verifier:** Prose, Markdown, code-span, adjacent-action, and whitespace-only examples pass shared checks plus literal-byte and inline-line-count assertions; run `cargo test -p yag-template-format region`.

**Human gate:** Review before/after samples from prose-containing templates.

**Commit:** `format: classify template regions safely`

## 6. Simple actions and delimiter padding

Lower ordinary actions, assignments, simple headers, and comments. Implement `DelimiterPadding::None` and `Spaces`; preserve trim marker spacing and leave unsupported shapes unchanged until later.

**Verifier:** Both padding modes, empty/comment/inline actions, and trim combinations pass output/reparse/fingerprint/idempotence checks; run `cargo test -p yag-template-format action`.

**Human gate:** Approve the ordinary, trim, empty, and comment delimiter matrix.

**Commit:** `format: format simple actions and delimiters`

## 7. Parsed block indentation

Lower `if`, `with`, `range`, `while`, `try`, `catch`, `else`, `end`, `define`, and `block` from AST nesting. Indent structural regions only; align `else` and `catch`; preserve protected inline sequences.

**Verifier:** Nested/empty/YAG-extension/else-catch fixtures pass shared checks; run `cargo test -p yag-template-format block`.

**Human gate:** Review nested real-template diffs.

**Commit:** `format: indent parsed template blocks`

## 8. Expressions, pipelines, and generic calls

Lower variables, fields, literals, receiver calls, parentheses, pipelines, and generic calls with `Group`, `SoftLine`, and `Nest(continuation_indent, ...)`. Do not add per-case width rules or simplify parentheses.

**Verifier:** Exact outputs just below/equal/above fit limits, including nested calls and block headers; run `cargo test -p yag-template-format expr`.

**Human gate:** Confirm renderer-driven breaks and consistent continuation indentation.

**Commit:** `format: wrap expressions with document groups`

## 9. Configurable key-value calls

Dispatch exact callee names through `FunctionLayouts`. Implement grouped `KeyValuePairs`: flat calls stay flat and broken calls use one continuation-indented key/value row each. Default odd counts retain generic call layout and report `OddKeyValueArgumentCount`.

```gotemplate
(sdict
  "a" "b"
  "c" "d"
)
```

**Verifier:** `dict`, `sdict`, a custom options-only name, nested values, fit boundaries, and odd counts pass `cargo test -p yag-template-format key_value`.

**Human gate:** Approve broken rows and dangling-value policy.

**Commit:** `format: support configurable key-value call layout`

## 10. CLI contract

Add stdout, `--check`, `--write`, stdin-filepath, width/indent/padding, and repeatable key-value-function flags. Reject incompatible check/write and stdin writes; only write explicitly named valid files; make exit statuses stable.

**Verifier:** Process tests cover stdout, check, write, conflict handling, invalid-input non-mutation, and custom layout flags.

**Human gate:** Approve flag names, exit codes, and write safety.

**Commit:** `format: add formatter command-line interface`

## 11. Corpus trial and regressions

Format a temporary copy of `scratch/code`; report total, parsed, changed, rejected, diagnostic counts, and longest line. Require reparse, fingerprint equality, and a stable second pass for every parseable file. Categorize diffs and turn surprises into fixtures before implementation changes.

**Verifier:** Repeatable corpus command or ignored integration test fails on parse/fingerprint/idempotence errors and prints the metrics.

**Human gate:** Review categorized diffs and rejected templates before real-file formatting or LSP work.

**Commit:** `format: add corpus regression coverage`

## 12. Integration and handoff

Run nightly `cargo fmt --all`, `cargo build --workspace`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets`; repeat corpus testing; document defaults, CLI behavior, non-goals, and known limits.

**Verifier:** All workspace commands pass with no corpus parse/fingerprint/idempotence failure for parseable templates.

**Human gate:** Release-readiness decision and separate decision on starting LSP integration.

**Commit:** `format: finalize formatter verification and documentation`

## Review checklist

- The commit has one purpose and includes boundary tests.
- Valid fixtures reparse, retain equal fingerprints, and are idempotent.
- Invalid input is not rewritten.
- Literal text and trim preservation remain explicit guarantees.
- Layout is explained by `Doc` and options, not a source-text heuristic.
