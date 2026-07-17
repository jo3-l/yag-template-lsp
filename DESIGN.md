# Better Function-Call Formatting

Status: draft

## Goals

Function-call layout should:

- keep short calls on one line;
- expose the logical structure of long variadic calls;
- use function signatures from the environment definitions when available;
- compose predictably with assignments, parentheses, nested calls, and action delimiters;
- remain deterministic, idempotent, and efficient in the existing Wadler-style pretty printer.

This design does not preserve user-inserted line breaks inside actions. As today, those breaks are formatter-owned whitespace.

## Proposed rules

### 1. Prefer one line

A call and its surrounding expression remain flat when their complete flat representation fits at the current column. “Complete” includes closing parentheses and the action's right delimiter.

For example:

```gotemplate
{{ $v := sdict "a" "b" }}
```

Source line breaks do not force expansion.

### 2. Hang only a trailing parenthesized call

Use one deliberately narrow exception to ordinary vertical layout. When an ordinary call's final argument is exactly a parenthesized, non-empty direct call, keep the outer callee, its preceding arguments, and the inner call's opening head together when that prefix fits:

```gotemplate
{{.Set "Out" (sdict
	"title" "💢 Deathmatch"
	"description" "..."
)}}
```

Here the two independently breakable segments are:

```text
.Set "Out" (sdict
                 ^ prefix ends after the inner callee
                  inner argument tail and `)` choose their own layout
```

This is the rule responsible for keeping `.Set "Out" (sdict` together; neither `.Set` nor `sdict` is special-cased. It applies only when:

- the outer call uses ordinary rather than signature-guided variadic layout;
- the final argument is syntactically `(` followed by a direct `FuncCall` or `ExprCall` with at least one argument and then `)`; and
- the prefix through the inner callee fits.

It does not apply to arbitrary breakable final arguments, pipelines, or a parenthesized atom. Restricting it to this syntactic shape both matches the motivating `.Set` example and avoids introducing general alternative-layout search.

If this rule is not applicable, keep the callee on its current line and put every argument on a continuation-indented row:

```gotemplate
{{ $out := print
	"All set! Every day at **"
	($target.Format "15:04 UTC")
	"**"
}}
```

This ordinary fallback applies to:

- calls to unknown functions;
- calls through an expression rather than a syntactic function name;
- non-variadic functions;
- variadic calls for which the special rules below cannot safely be applied.

An argument is itself a document, not an indivisible string. A nested call may remain flat on its row if it fits, or choose its own expanded layout if it does not.

### 3. Keep the fixed prefix of a variadic call together when possible

For a known function whose final parameter is variadic, divide the arguments into:

- a fixed prefix, containing the callee and the arguments bound to non-variadic parameters; and
- a variadic tail.

When the whole call must expand, first try to keep the fixed prefix flat at its current column. Put each variadic argument on a separate continuation row.

Given `parseArgs(numRequired, errorMsg, argDefs...)`:

```gotemplate
{{ $args := parseArgs 1 "foo bar"
	(carg ...)
	(carg ...)
	(carg ...)
}}
```

If the fixed prefix does not fit, it uses the ordinary expanded-call layout before the variadic rows are emitted:

```gotemplate
{{ $args := parseArgs
	1
	"a description which does not fit beside the callee"
	(carg ...)
	(carg ...)
}}
```

This rule also naturally handles functions such as `joinStr(sep, args...)`, `index(item, keys...)`, and `addMessageReactions(channel, messageID, emojis...)`.

The split is positional. If there are fewer arguments than fixed parameters, there is no variadic tail to format specially. Optional fixed parameters do not require a separate rule because template calls cannot omit a positional argument in the middle of the list.

### 4. Key/value variadic tails use one pair per row

Environment definitions do not currently carry a rich parameter type or layout annotation. Retain the existing narrow convention that a variadic parameter named exactly `opts` or `keyvalues` denotes alternating key/value arguments.

If that tail contains an even number of arguments, place one pair on each expanded row. This now applies whether or not fixed parameters precede the variadic parameter.

Given `sdict(keyvalues...)`:

```gotemplate
{{ $a := sdict
	"a" "b"
	"c" "d"
}}
```

Given `editThread(thread, opts...)`:

```gotemplate
{{ editThread .Thread
	"slowmode" 10
	"auto_archive_duration" 1440
}}
```

The single space within a pair is not a candidate row break. Either member may contain a nested document which expands internally.

If the key/value tail has odd cardinality, use the ordinary expanded-call layout for the entire call. Guessing pairs in malformed or intentionally unusual input would make the layout misleading.

### 5. Expanded parentheses get a closing row

Parentheses stay attached to the expression they open. If a parenthesized expression expands, its `)` is placed on a separate row at the parenthesized expression's indentation level:

```gotemplate
{{ $a := (sdict
	"a" "b"
	"c" "d"
) }}
```

“Aligned” here means aligned to the structural indentation level of the row containing the opening `(`, not to the opening character's visual column. This works for both tab and space indentation and avoids alignment becoming unstable when a prefix changes length.

Each nested parenthesized expression owns its own closing row:

```gotemplate
{{.Set "Out" (sdict
	"title" "💢 Deathmatch"
	"description" (joinStr "\n" .GameData.Msgs.StringSlice)
	"color" 14232643
	"fields" (cslice
		(sdict "name" $player0.User.Username "value" (print $player0.HP "/100 HP") "inline" true)
		(sdict "name" $player1.User.Username "value" (print $player1.HP "/100 HP") "inline" true)
	)
)}}
```

In this example delimiter padding is configured as `none`. The other examples use `spaces`.

### 6. A multiline action ends with a closing row

The left delimiter and the start of the action body stay on the same row. If the action body becomes multiline, the right delimiter must not remain after an ordinary final argument:

```gotemplate
{{ $a := sdict
	"a" "b"
	"c" "d"
}}
```

Not:

```gotemplate
{{ $a := sdict
	"a" "b"
	"c" "d" }}
```

If the body's last physical row was generated solely to close a trailing parenthesized expression, the action delimiter joins that existing closing row rather than creating a second one:

```gotemplate
{{ $a := (sdict
	"a" "b"
	"c" "d"
) }}
```

With delimiter padding set to `none`, the same suffix is `)}}`. With a right trim marker it is `) -}}`. Thus nested syntactic delimiters each receive a row, while the template delimiter acts as the outer suffix of the final syntactic closing row.

Opening delimiter padding is fixed horizontal whitespace, not a break opportunity:

- `spaces`: `{{ expression`;
- `none`: `{{expression`;
- a left trim marker retains its required `{{- ` spelling.

This intentionally replaces the current style which can put `{{` alone on the first row of a multiline action.

## Compact statement of the layout algorithm

For a function call:

1. Construct its flat form.
2. Classify it from the exact syntactic callee name and its environment definition.
3. If it fits, render the flat form.
4. Otherwise:
   - for an even `opts...` or `keyvalues...` tail, emit one pair per row;
   - for another variadic tail, keep the fixed prefix flat if possible and emit one tail argument per row;
   - for an ordinary call ending in a non-empty parenthesized call, use the segmented trailing-call layout;
   - otherwise, emit one argument per row.
5. Let every argument independently choose flat or expanded layout within its row.

For delimiters:

1. An expanded parenthesized expression creates a row for its `)`.
2. A multiline action creates a row for `}}` unless its trailing parenthesized expression already created a closing row.
3. Closing rows use structural indentation, never visual-column alignment.

These rules are based on syntax and environment metadata, not on source whitespace or hard-coded function names.

## Pretty-printing design

### Documents for calls

`Group`, `SoftLine`, and `Indent` remain sufficient for ordinary vertical argument rows.

An ordinary vertical call has the conceptual shape:

```text
group(
  callee
  + indent((soft_line + argument)*)
)
```

A variadic call adds an independently grouped fixed prefix:

```text
group(
  group(callee + indent((soft_line + fixed_argument)*))
  + indent((soft_line + variadic_row)*)
)
```

When the outer group is flat, all separators become spaces. When it breaks, the variadic rows break while the nested prefix group gets its own fit decision. This is directly supported by the renderer's current independent nested-group semantics.

A key/value row is `key + " " + value`; a normal row is one argument document.

The restricted trailing-parenthesized-call rule can be represented without an ordered choice. Split the final argument immediately after its inner callee and give the outer prefix and inner argument tail independent groups:

```text
prefix_group[id](
  outer_callee
  + indent((soft_line + preceding_argument)*)
  + indent(soft_line + "(" + inner_callee)
)
+ indent_if_break[id](inner_argument_tail_and_closing_paren)
```

If both groups fit, this is the ordinary flat call. If the prefix fits but the inner tail does not, only the inner call expands, producing `.Set "Out" (sdict` on the first row. If the prefix does not fit, its soft lines produce the ordinary outer argument rows. `indent_if_break` gives the inner tail one additional structural indentation level in that last case so it remains nested under the relocated `(sdict` head.

This decomposition is particularly simple for this template language because calls are whitespace-separated and an outer call has no own parentheses to coordinate. It prioritizes the trailing call's internal breaks by construction rather than asking the renderer to compare several complete renderings.

### Conditional closing boundaries

The current algebra cannot express “nothing (or one padding space) when flat, newline when this particular group breaks.” `SoftLine` is insufficient because it always becomes one space in flat mode and only observes the ambient mode.

Add group identities and an `IfBreak` document, conceptually:

```text
Group { id, doc }
IfBreak { group_id, broken, flat }
```

The renderer records the flat/broken decision for each group. `IfBreak` selects a branch using the named group's decision. During a flat `fits` probe, it follows the flat branch. Group IDs can be dense integers allocated while lowering, so recording and looking up decisions is O(1).

Named break-condition documents are not part of Wadler's minimal algebra, but there are direct production precedents: Prettier's `ifBreak(..., { groupId })` and Biome's `if_group_breaks(...).with_group_id(...)` both select content from an already printed named group's mode. This avoids inspecting rendered strings or performing a second formatting pass.

Prettier and Biome implement broader “group the final argument” rules with ordered alternatives (`conditionalGroup` and `BestFitting`, respectively). Both document these facilities as expensive or last-resort mechanisms, and Biome adds caching in its call-argument implementation to avoid quadratic behavior for nested grouped arguments. The restricted segmented layout above deliberately avoids importing that machinery. Named `IfBreak` has constant-time mode lookup and does not introduce alternative search.

A parenthesized expression uses an empty flat boundary and a newline broken boundary before `)`:

```text
group[id](
  "(" + expression + if_break[id](line, empty) + ")"
)
```

The action's closing boundary needs to know both the action group's decision and whether its trailing parenthesis group broke:

```text
if trailing_parenthesis_group broke:
  configured closing padding
else if action_group broke:
  line
else:
  configured closing padding
```

The same rule works when closing padding is empty or is the grammar-required space before `-}}`.

### Tracking the trailing closing group

Expression lowering should carry a small amount of layout metadata alongside each `Doc`:

```text
ExpressionDoc {
  doc,
  trailing_closing_group: Option<GroupId>,
}
```

The metadata means “if this group breaks, the expression ends on that group's generated closing row.” It propagates through wrappers which do not append output:

- an assignment inherits it from its value;
- a call inherits it from its final argument;
- a pipeline inherits it from its final stage;
- a keyword or other prefix preserves it;
- a parenthesized expression replaces it with its own group ID.

A suffix such as `.Field` clears it because the expression no longer ends at the parenthesis. Literals and bare calls have no closing group of their own.

This is enough to distinguish the desired `.Set ... (sdict ...)` case from a bare `sdict ...` call without coupling `action.rs` to call internals. The cross-module API should expose only a focused formatted-expression fragment (document plus trailing closing-group ID), in keeping with the rules-module boundary guidelines.

## Function classification details

Only `FuncCall` nodes with an exact name lookup can use environment-guided layout. `ExprCall` nodes use ordinary layout.

For a known function:

1. Check whether its final parameter is variadic.
2. The number of fixed arguments is the number of preceding parameters.
3. If the variadic parameter is named `opts` or `keyvalues`, use pair rows only when the actual tail length is even.
4. Otherwise use one variadic argument per row.

No names besides the two established parameter conventions are special-cased. In particular, `parseArgs` is not special: its desired layout follows from `argDefs...` being variadic.

## Alternatives rejected

### Always put every argument on its own row

This is simple, but loses useful signature structure in calls such as `parseArgs 1 "description" argDefs...` and `joinStr "\n" values...`.

### Preserve source grouping

Source whitespace inside an action is not semantic and would make formatting non-canonical and potentially non-idempotent.

### Keep `}}` after the last argument

This produces the explicitly undesirable `"c" "d" }}` form and makes the end of a long action hard to scan.

### Always give `}}` a separate row

That produces two adjacent closing rows for parenthesized calls:

```gotemplate
)
}}
```

Coalescing the template suffix with an existing final parenthesis row is both more compact and consistent with `) }}` in flat actions.

### Infer key/value rows from argument syntax

Keys are not required to be string literals, and values can have any expression shape. Signature metadata is more stable. Without richer environment annotations, the existing `opts`/`keyvalues` naming convention is the least surprising boundary.

### Decide delimiter placement after rendering

Scanning or rewriting rendered text would violate the document model, complicate indentation, and require extra work. Named group decisions express the condition directly during one render.

## Test plan

Add focused snapshots covering:

1. flat and expanded `sdict` calls;
2. an ordinary call which hangs a trailing parenthesized call, plus a prefix-too-wide fallback and ineligible final-expression cases;
3. an expanded bare `sdict` action with `}}` on its own row;
4. parenthesized `sdict` with `) }}` on one closing row;
5. `parseArgs` with a flat fixed prefix and expanded `argDefs` tail;
6. `parseArgs` whose fixed prefix also needs to expand;
7. a function with fixed arguments followed by even `opts...` pairs;
8. an odd key/value tail falling back to ordinary layout;
9. the nested `.Set`/`sdict`/`cslice` example;
10. both delimiter-padding modes and right trim delimiters;
11. exact width boundaries where a closing parenthesis or action delimiter changes the fit decision;
12. a parenthesized expression followed by a field suffix, which must not be treated as a trailing closing row;
13. idempotence of every expanded form.

Add unit tests for `IfBreak` selection and nested named groups in the pretty renderer. Existing snapshot checks will also verify parsing, semantic fingerprints, and idempotence.

## Expected implementation scope

The implementation should be confined primarily to:

- `src/pretty/mod.rs`: group IDs and `IfBreak`;
- `src/pretty/render.rs`: group-decision recording and conditional branch selection;
- `src/rules/expr.rs`: call classification, variadic prefix/row layout, parenthesized layout, and trailing-closing metadata;
- `src/rules/action.rs`: fixed opening padding and conditional right-delimiter placement;
- focused formatter and renderer tests.

No parser or environment-definition schema change is required.
