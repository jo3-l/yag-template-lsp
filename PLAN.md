# Function-Call Formatting Implementation Plan

This plan implements `DESIGN.md` as a sequence of small, independently reviewable changes. Every slice should compile, keep the formatter idempotent, and include focused tests for any behavior it changes.

## Scope decisions

The implementation will:

- add named group decisions and conditional documents to the existing pretty printer;
- format expanded parentheses and action delimiters as closing rows;
- classify known calls from envdef signatures;
- keep fixed arguments together ahead of variadic rows;
- format even `opts...` and `keyvalues...` tails as pairs;
- hang only a final argument shaped as a non-empty parenthesized direct call;
- implement that hanging layout with independently grouped call segments, not `Choice`, `conditionalGroup`, or general backtracking.

It will not:

- change the parser or envdef schema;
- preserve source line breaks inside actions;
- infer key/value structure from argument syntax;
- add a general alternative-layout primitive;
- apply hanging layout to arbitrary pipelines, parenthesized atoms, or other breakable expressions.

## Cross-cutting invariants

Each slice must preserve these properties:

1. Parse-invalid input is returned unchanged.
2. Formatting preserves the semantic fingerprint.
3. Formatting is idempotent.
4. Trim markers and literal token text are unchanged.
5. Anonymous groups retain their current independent fit behavior.
6. Named group lookup is constant time and does not inspect rendered text.
7. Rules remain syntax-driven; only exact envdef parameter names `opts` and `keyvalues` have special meaning.

---

## Slice 1: Add named conditional groups to the document algebra

**Suggested commit:** `format: add named conditional documents`

### Purpose

Introduce the renderer capability needed by closing delimiters and conditional indentation without changing formatter output.

### Changes

In `src/pretty/mod.rs`:

- Add an opaque, copyable `GroupId` backed by a dense integer.
- Preserve the existing anonymous `group(doc)` helper.
- Add a named-group constructor, for example `group_with_id(id, doc)`.
- Add `IfBreak { group_id, broken, flat }` and a focused constructor.
- Add `IndentIfBreak { group_id, indent, doc }`, or an equivalent non-duplicating helper, for the segmented trailing-call layout.
- Keep these types and helpers crate-private.

In `src/lower.rs`:

- Add a monotonically increasing group-ID allocator to `Formatter`.
- Expose one narrow `new_group_id()` method to formatter rules.

In `src/pretty/render.rs`:

- Record the selected mode of named groups in a dense `Vec<Option<Mode>>` indexed by `GroupId`.
- Record a group's decision before rendering its contents, allowing an `IfBreak` inside that group to reference it.
- During a flat fit probe, select the flat branch for a group whose actual decision is not yet available.
- During actual rendering, require referenced groups to be the current enclosing named group or an already rendered group; treat an unresolved reference as an internal document-construction error.
- Make `IndentIfBreak` affect indentation only when its named group selected broken mode.

### Tests

Add renderer unit tests for:

- flat and broken branches of `IfBreak`;
- an `IfBreak` referring to its enclosing named group;
- an `IfBreak` referring to an earlier sibling group;
- conditional indentation in flat and broken modes;
- nested named groups retaining independent decisions;
- anonymous groups producing byte-identical output to the current implementation.

### Review checkpoint

- The formatter snapshot suite must have no changes.
- The renderer still performs one output traversal plus bounded fit probes.
- Group IDs are dense and are not exposed through the formatter's public API.

### Verification

```sh
cargo test -p yag-template-format pretty
cargo test -p yag-template-format --test format_snapshots
```

---

## Slice 2: Carry trailing-closure metadata through expression lowering

**Suggested commit:** `format: track expression closing groups`

### Purpose

Create the narrow expression/action boundary needed to coalesce `)}}` with a generated parenthesis-closing row. This is a representation refactor only; output should remain unchanged.

### Changes

In `src/rules/expr.rs`:

- Introduce a focused crate-private result type:

  ```rust
  struct ExpressionDoc {
      doc: Doc,
      trailing_closing_group: Option<GroupId>,
  }
  ```

- Convert recursive expression lowering to return `ExpressionDoc` internally.
- Provide small constructors/helpers for:
  - a plain fragment with no trailing closing group;
  - prefixing without changing trailing metadata;
  - concatenating a suffix which clears trailing metadata;
  - extracting the underlying `Doc` when metadata is irrelevant.
- Propagate metadata according to `DESIGN.md`:
  - assignment from its value;
  - call from its final argument;
  - pipeline from its final stage;
  - prefixes without appended output preserve it;
  - field suffixes clear it.
- Do not yet assign a group to parentheses; all current expressions may still carry `None` in this slice.

In `src/rules/action.rs`:

- Adapt expression consumers to accept the new focused result type and discard unused metadata for now.
- Do not change delimiter construction yet.

### Tests

- Run every existing snapshot unchanged.
- Add small `expr.rs` unit tests for metadata propagation if this can be done without exposing implementation details broadly; otherwise rely on focused behavior tests in Slice 3.

### Review checkpoint

- `action.rs` learns only about `ExpressionDoc`, not call classification or parenthesis internals.
- No new dependency is introduced between `action.rs` and `expr.rs` beyond this narrow return type.
- Snapshot output is byte-identical.

### Verification

```sh
cargo test -p yag-template-format --test format_snapshots
cargo test -p yag-template-format
```

---

## Slice 3: Implement coherent parenthesis and action closing rows

**Suggested commit:** `format: put multiline delimiters on closing rows`

### Purpose

Implement the delimiter behavior as one coherent change so the tree never passes through an intermediate state that emits both `)` and `}}` on separate adjacent rows unintentionally.

### Changes

In `src/rules/expr.rs`:

- Give each parenthesized expression a named group.
- Place `if_break(group_id, line(), empty())` immediately before `)`.
- Set `trailing_closing_group` to that parenthesis group's ID.
- Ensure a field suffix after a parenthesized expression clears the metadata.
- Ensure nested parentheses each own their own group and structural closing indentation.

In `src/rules/action.rs`:

- Allocate a named group for each formatted action.
- Make opening padding fixed horizontal text:
  - ordinary + spaces: one literal space;
  - ordinary + none: empty;
  - left trim: the grammar-required literal space.
- Build the right boundary from two conditions:
  1. if the expression's trailing parenthesis group broke, use normal closing padding and attach the action delimiter to that row;
  2. otherwise, if the action group broke, insert a line before the right delimiter;
  3. otherwise, use normal flat closing padding.
- Preserve the grammar-required space before `-}}`.
- Keep non-expression actions on the same delimiter path by giving their body fragment no trailing closing group.

### Focused snapshots

Add new pairs under `tests/snapshots/action/` or `tests/snapshots/expr/` for:

- bare expanded `sdict` ending with `}}` on its own row;
- parenthesized expanded `sdict` ending with `) }}`;
- delimiter padding `none`, producing `)}}`;
- right trim delimiter, producing `) -}}` or a standalone `-}}` row as appropriate;
- nested parentheses with one structural closing row per nesting level;
- a parenthesized expression followed by a field suffix;
- exact width boundaries where only the closing suffix changes the fit decision.

Update existing snapshots only where the new opening/closing policy intentionally changes output. Inspect every updated fixture rather than accepting a bulk snapshot rewrite.

### Review checkpoint

Confirm these representative forms exactly:

```gotemplate
{{ $a := sdict
	"a" "b"
	"c" "d"
}}
```

```gotemplate
{{ $a := (sdict
	"a" "b"
	"c" "d"
) }}
```

The opening delimiter must not move to a row by itself, and parenthesized calls must not produce:

```gotemplate
)
}}
```

### Verification

```sh
cargo test -p yag-template-format --test format_snapshots
cargo test -p yag-template-format
```

---

## Slice 4: Introduce signature-driven call classification

**Suggested commit:** `format: classify variadic call layouts`

### Purpose

Separate envdef interpretation from document construction before changing variadic output.

### Changes

In `src/rules/expr.rs`:

- Replace `function_uses_key_value_layout` with a private classification model, conceptually:

  ```rust
  enum CallLayout {
      Ordinary,
      Variadic {
          fixed_count: usize,
          rows: VariadicRows,
      },
  }

  enum VariadicRows {
      Arguments,
      KeyValuePairs,
  }
  ```

- Classify only exact syntactic `FuncCall` names found in envdefs.
- Treat `ExprCall` as ordinary.
- Derive `fixed_count` from parameters before the final variadic parameter.
- Select `KeyValuePairs` only for final variadic parameters named exactly `opts` or `keyvalues`.
- Resolve odd actual key/value tails to `Ordinary` before constructing rows.
- Treat calls with too few arguments to reach the variadic tail as ordinary/fixed-only calls.

Initially use the classifier to replace the old key/value predicate while deliberately preserving its current output gate: only an even pair tail with `fixed_count == 0` uses the existing pair builder in this slice. Slice 5 removes that temporary compatibility gate when it adds the fixed-prefix document shape. This keeps the classification independently reviewable without changing snapshots.

### Tests

Use custom `EnvDefs` in unit tests to cover:

- unknown and expression callees;
- non-variadic functions;
- `args...` with no fixed prefix;
- `parseArgs(first, description, argDefs...)`;
- `mixed(first, opts...)`;
- even and odd key/value tails;
- fewer actual arguments than fixed parameters.

### Review checkpoint

- Classification contains no document construction.
- No function name such as `sdict` or `parseArgs` is hard-coded.
- Existing snapshots remain unchanged.

### Verification

```sh
cargo test -p yag-template-format
cargo test -p yag-template-format --test format_snapshots
```

---

## Slice 5: Format fixed prefixes and variadic rows

**Suggested commit:** `format: structure variadic call rows`

### Purpose

Implement the signature-guided call layouts independently of the special trailing-parenthesized-call rule.

### Changes

In `src/rules/expr.rs`:

- Refactor call construction around a private call document representation which keeps the callee head separate from its argument tail. This representation should be reusable by Slice 6.
- Keep ordinary calls on their existing one-argument-per-row layout.
- For a variadic call with actual tail arguments:
  - independently group `callee + fixed arguments`;
  - keep that prefix flat when it fits at the current column;
  - put each variadic row under one continuation indent.
- For `VariadicRows::Arguments`, emit one tail argument per row.
- For `VariadicRows::KeyValuePairs`, emit one pair per row with one fixed space inside the pair.
- Preserve independent nested argument groups.
- Keep odd key/value tails on the ordinary call path.

Avoid separate width arithmetic. All decisions must come from groups and fit probes.

### Focused snapshots

Add fixtures for:

- `parseArgs` with a flat fixed prefix and multiple `argDefs` rows;
- `parseArgs` whose fixed prefix itself must break;
- `joinStr(sep, args...)` showing the fixed separator beside the callee;
- `editThread(thread, opts...)` showing fixed arguments plus option pairs;
- even `sdict(keyvalues...)` pairs;
- odd `sdict` arguments falling back to ordinary rows;
- nested values which independently remain flat or expand.

Add custom-envdef unit coverage for cases unavailable in bundled definitions.

### Review checkpoint

- Short calls remain byte-identical and flat.
- `parseArgs` follows its signature rather than a callee-name exception.
- A pair is never split between its key and value by the outer call layout.
- Calls with zero actual variadic arguments do not acquire a spurious row.

### Verification

```sh
cargo test -p yag-template-format --test format_snapshots
cargo test -p yag-template-format
```

---

## Slice 6: Hang a trailing parenthesized direct call

**Suggested commit:** `format: hang trailing parenthesized calls`

### Purpose

Implement the restricted `.Set "Out" (sdict ...)` behavior without adding general layout alternatives.

### Eligibility

Apply the segmented layout only when all are true:

1. The outer call uses `CallLayout::Ordinary`.
2. Its final argument is `Expr::Parenthesized`.
3. The parenthesized inner expression is directly a non-empty `FuncCall` or `ExprCall`.
4. No syntax follows the closing parenthesis within that argument.

Pipelines, field-suffixed parentheses, parenthesized atoms, empty calls, and signature-guided variadic outer calls use existing layouts.

### Changes

In `src/rules/expr.rs`:

- Add a private helper which lowers an eligible final argument into exactly two pieces:
  - `head`: `(` plus the inner callee;
  - `tail`: the inner arguments plus the parenthesis-closing boundary.
- Construct a named outer-prefix group containing:
  - the outer callee;
  - preceding outer arguments separated by soft lines;
  - one final soft line followed by the inner `head`.
- Append the independently grouped inner `tail` outside the prefix group.
- Apply `IndentIfBreak(prefix_group_id, continuation_indent, tail)` so the tail remains nested if the outer prefix itself breaks.
- Preserve the final parenthesis's `trailing_closing_group` so the action delimiter can join its closing row.
- Reuse the call-head/tail builders from Slice 5; do not format the inner AST twice.

### Focused snapshots

Add fixtures for:

- the complete `.Set` / `sdict` / `cslice` motivating example;
- a flat `.Set "Out" (sdict ...)` call;
- a fitting prefix with an expanded trailing call;
- a prefix too wide to remain on one row;
- nested trailing parenthesized calls;
- each ineligible shape listed above, confirming ordinary fallback;
- both tabs and configured continuation spaces.

### Review checkpoint

The implementation must contain no `Choice`, `BestFitting`, render backtracking, rendered-text inspection, or duplicated lowering of the final argument.

Confirm the desired nesting:

```gotemplate
{{.Set "Out" (sdict
	"fields" (cslice
		(sdict ...)
		(sdict ...)
	)
)}}
```

If the prefix cannot fit, verify that the inner tail receives one additional structural continuation level relative to the relocated inner head.

### Verification

```sh
cargo test -p yag-template-format --test format_snapshots
cargo test -p yag-template-format
```

---

## Slice 7: Consolidate regression coverage and verify performance properties

**Suggested commit:** `format: cover nested call layouts`

### Purpose

Review the combined behavior, remove temporary helpers, and ensure the feature remains stable across the formatter corpus.

### Changes

- Consolidate duplicated call-row helpers introduced during earlier slices.
- Keep all representation types private to `expr.rs` unless they are part of the narrow `ExpressionDoc` boundary.
- Add comments documenting:
  - why named groups must precede their external `IfBreak` references;
  - why the trailing-call rule is syntactically restricted;
  - why no general `Choice` is used.
- Update `DESIGN.md` status and details only if implementation discoveries changed the accepted design.
- Do not add configuration options for these rules.

### Final regression checks

Verify:

- all focused snapshots from `DESIGN.md`;
- every pre-existing snapshot, with intentional changes reviewed individually;
- corpus formatting and fingerprint tests;
- idempotence on deeply nested calls;
- flat width boundaries with both tabs and spaces;
- malformed/parse-invalid input behavior;
- no pathological growth from nested trailing calls (there is no alternatives search).

### Required final verification

```sh
cargo test -p yag-template-format --test format_snapshots
cargo test -p yag-template-format
cargo test --workspace
cargo +nightly fmt --all --check
cargo clippy --workspace --all-targets
```

If formatting is required:

```sh
cargo +nightly fmt --all
cargo +nightly fmt --all --check
```

## Handoff checklist

- [ ] Every behavior change has a new focused snapshot pair.
- [ ] Snapshot updates outside the new fixtures are listed and explained.
- [ ] The full motivating examples match `DESIGN.md` exactly.
- [ ] No parser or envdef schema changes were needed.
- [ ] No general alternative-layout primitive was introduced.
- [ ] All verification commands and any unavailable checks are reported.
