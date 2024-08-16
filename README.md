# YAGPDB Template LSP

A language server for the [YAGPDB](https://yagpdb.xyz) templating language, with accompanying
extensions published on the VSCode marketplace and Open VSX.

## Features

This project is in the MVP stage and so only implements the most basic of LSP features, namely,

- [x] error reporting;
- [x] basic code completion for variable and function names.

However, while the current set of features is minimal, the sound technical foundation means many
features should be trivial to add—for instance, inlay hints for function parameter names, code
folding, and semantic highlighting should all be straightforward to add.

## Roadmap

The following are relatively straightforward changes that are planned in the near future:

- [ ] TextMate syntax highlighting
- [ ] Documentation for all template functions
- [ ] Semantic tokens (i.e., more accurate syntax highlighting)
- [ ] Inlay hints for function parameter names
- [ ] Code folding

In the long term, we would also like to implement:

- [ ] Lints for common code issues
- [ ] Type-aware code completion
  - Type-checking is almost fully implemented in the [`feat/typechecking`][typeck-branch],
    save function overload resolution and integration into the language server.

[typeck-branch]: https://github.com/jo3-l/yag-template-lsp/tree/feat/typechecking

## Technical overview

The primary contribution of this project is a language server implemented in Rust and split over
three crates:

- [`yag-template-syntax`][syntax-crate-dir], which provides an error-resilient parser for the YAGPDB
  templating language that outputs an untyped CST using the
  [Rowan](https://github.com/rust-analyzer/rowan) library and a typed AST view of the syntax tree;
- [`yag-template-analysis`][analysis-crate-dir], which only provides basic symbol resolution at the moment;
- [`yag-template-lsp`][lsp-crate-dir], which implements the actual language server protocol using
  [tower-lsp](https://github.com/ebkalderon/tower-lsp).

A VSCode extension powered by this language server is provided in the [`editors/vscode`][editors-vscode-dir] directory.
Contributions for other editors are welcome.

[syntax-crate-dir]: https://github.com/jo3-l/yag-template-lsp/tree/main/crates/yag-template-syntax
[analysis-crate-dir]: https://github.com/jo3-l/yag-template-lsp/tree/main/crates/yag-template-analysis
[lsp-crate-dir]: https://github.com/jo3-l/yag-template-lsp/tree/main/crates/yag-template-lsp
[editors-vscode-dir]: https://github.com/jo3-l/yag-template-lsp/tree/main/editors/vscode

### Inspiration

We stand on the shoulders of giants here; the structure of this project is heavily informed by
[rust-analyzer](https://github.com/rust-lang/rust-analyzer) (and matklad's excellent blog posts),
[typst-lsp](https://github.com/nvarner/typst-lsp), and [rhai's LSP](https://github.com/rhaiscript/lsp).
This project would not be possible without this exceptional prior work—thank you to everyone who
contributed to the above projects!

## License

This project is released under the MIT license.
