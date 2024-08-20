# YAGPDB Template LSP

<a href="https://marketplace.visualstudio.com/items?itemName=jo3-l.yag-template-lsp"><img src="https://img.shields.io/visual-studio-marketplace/v/jo3-l.yag-template-lsp?style=for-the-badge&label=VSCode" alt="Visual Studio Marketplace Version"></a>
<a href="https://open-vsx.org/extension/jo3-l/yag-template-lsp"><img src="https://img.shields.io/open-vsx/v/jo3-l/yag-template-lsp?style=for-the-badge&color=blue" alt="Open VSX Version"></a>
<a href="https://github.com/jo3-l/yag-template-lsp/actions"><img src="https://img.shields.io/github/actions/workflow/status/jo3-l/yag-template-lsp/ci.yml?style=for-the-badge" alt="GitHub build status"></a>

A language server for the [YAGPDB](https://yagpdb.xyz) templating language, with accompanying
extensions published on the VSCode marketplace and Open VSX.

## Features

A range of basic LSP features are already implemented, namely,

- Syntax highlighting
- Error reporting as you type
- Basic code completion for variable and function names
- Goto definition and find all references for variables
- Hover documentation for functions
- Inlay hints for function parameter names
- Automatic indentation
- Folding ranges

## Roadmap

The following are relatively straightforward changes that are planned for the near future:

- [ ] Automated variable renaming

In the medium term, we would also like to implement:

- [ ] Lints for common code issues, e.g., checking `printf` format verbs
- [ ] Type-aware code completion
  - A partial type-checker implementation is in the [`feat/typechecking`][typeck-branch] branch,
    save function overload resolution and language server integration.

Finally, as a low-priority, long-term goal, we hope to better modularize and test the language
server implementation (and indeed the project as a whole.)

[typeck-branch]: https://github.com/jo3-l/yag-template-lsp/tree/feat/typechecking

## Architecture overview

The project comprises three main components:

- a language server and associated static analysis tooling under the [`crates`][crates-dir] directory, written in Rust;
- structured YAGPDB function definitions and documentation under the [`bundled-defs`][bundled-defs-dir] directory;
- and a VSCode extension powered by the Rust language server under the [`editors/vscode`][editors-vscode-dir] directory,
  written in TypeScript. (Contributions for other editors are welcome.)

Of the above, the most interesting (and where the bulk of the complexity lies) is the Rust component, which is itself
split into the following crates:

- [`yag-template-syntax`][syntax-crate-dir], which provides an error-resilient parser for the YAGPDB templating language
  outputting a CST using the [Rowan](https://github.com/rust-analyzer/rowan) library, in addition to a typed AST view of
  the syntax tree;
- [`yag-template-analysis`][analysis-crate-dir], which provides basic symbol resolution;
- [`yag-template-envdefs`][envdefs-crate-dir], which provides a parser for template function definitions and embeds
  the definitions under `bundled-defs`; and last but not least,
- [`yag-template-lsp`][lsp-crate-dir], which implements the actual language server protocol
  using [tower-lsp](https://github.com/ebkalderon/tower-lsp).

[crates-dir]: https://github.com/jo3-l/yag-template-lsp/tree/main/crates/
[bundled-defs-dir]: https://github.com/jo3-l/yag-template-lsp/tree/main/bundled-defs/README.md
[syntax-crate-dir]: https://github.com/jo3-l/yag-template-lsp/tree/main/crates/yag-template-syntax
[analysis-crate-dir]: https://github.com/jo3-l/yag-template-lsp/tree/main/crates/yag-template-analysis
[envdefs-crate-dir]: https://github.com/jo3-l/yag-template-lsp/tree/main/crates/yag-template-envdefs
[lsp-crate-dir]: https://github.com/jo3-l/yag-template-lsp/tree/main/crates/yag-template-lsp
[editors-vscode-dir]: https://github.com/jo3-l/yag-template-lsp/tree/main/editors/vscode

### Inspiration

We stand on the shoulders of giants. `yag-template-lsp` is heavily inspired by—and indeed, would not exist without—the
following excellent projects:

- [rust-analyzer](https://github.com/rust-lang/rust-analyzer) and [matklad's excellent blog
  posts](https://matklad.github.io/);
- [typst](https://github.com/typst/typst) and [typst-lsp](https://github.com/nvarner/typst-lsp);
- [rhai's LSP](https://github.com/rhaiscript/lsp);
- and [RSLint](https://github.com/rslint/rslint).

## Contributing

Contributions are very welcome, though familiarity with Rust, error-tolerant parsers, and the
language server protocol is a prerequisite for any significant additions. If you are interested,
please feel free to ping me in the `#programming-discussion` channel of the YAGPDB community server
for guidance.

### Development tips

**Requirements:** recent version of Node.js, stable Rust toolchain, and nightly rustfmt.

The most straightforward way to run a modified version of the language server is to open this
project in VSCode and use the provided `Run Extension` debug configuration. This will compile both
the Rust and TypeScript components and open a new VSCode window with the modified language server
installed.

To debug changes, use the logging macros from the `tracing` crate. By default, however, only error logs are output. Add
the following entries to your `settings.json` to show more:

```jsonc
{
	"yag-template-lsp.trace.server": "messages", // "trace" for all LSP interactions
	"yag-template-lsp.server.extraEnv": {
		// hide tower_lsp tracing logs (which are redundant with the above), but display everything else
		"YAG_LSP_LOG": "tower_lsp::codec=info,trace"
	}
}
```

The resulting logs will then be visible in the VSCode output window under the `YAGPDB Template Language Server` channel.

## License

This project is released under the MIT license.
