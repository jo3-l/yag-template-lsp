# Formatter snapshots

Each `.in` file is a complete formatter input. It may begin with YAML front
matter that configures the supported `FormatOptions` fields; without that
block, it uses the formatter defaults. Its sibling `.out` file is the
formatter output exactly as written, with no header or newline normalization.

Supported front-matter fields are `max_width`, `indent`,
`continuation_indent`, and `delimiter_padding`. Indentation is `tabs` or
`{ spaces: <positive width> }`; delimiter padding is `none` or `spaces`.
Omitted fields use the formatter defaults. Function layouts are deliberately
not configurable in fixtures.

Run `YAG_UPDATE_SNAPSHOTS=1 cargo test -p yag-template-format --test
format_snapshots` to create or update snapshots. Ordinary test runs never
modify them.
