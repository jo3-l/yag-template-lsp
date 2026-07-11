# Formatter fixture inventory

These are deliberately small, valid templates extracted from representative
patterns in `scratch/corpus`. They provide the initial regression inputs; later
milestones add expected output and preservation assertions alongside them.

| Fixture | Covered shape | Corpus source |
| --- | --- | --- |
| `inline-prose.tmpl` | Inline prose and field action | `scratch/corpus/code_snippets/get_username_color.go.tmpl` |
| `blocks.tmpl` | Block with `else` | `scratch/corpus/fun/counting/basic/counting.go.tmpl` |
| `pipeline.tmpl` | Expression pipeline | `scratch/corpus/utilities/json.go.tmpl` |
| `comments.tmpl` | Comment-only action | `scratch/corpus/moderation/notes/notes.go.tmpl:2` |
| `trim-markers.tmpl` | Left and right trim markers | `scratch/corpus/moderation/staff_on_duty.go.tmpl:26` |
| `compressed.tmpl` | Adjacent actions and YAG `try`/`catch` | `scratch/corpus/fun/counting/advanced/counting_v2.go.tmpl:176` |
| `key-value-calls.tmpl` | `dict` and `sdict` | `scratch/corpus/moderation/raid_guard/raid_admin.go.tmpl:11,30` |

The corpus remains read-only during formatter development. Corpus trials use a
temporary copy only, as specified by the formatter milestones.
