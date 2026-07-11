# Repository Guidelines

## Project Structure & Module Organization

This is a Rust workspace with a VS Code extension wrapper. Core crates live under `crates/`: `yag-template-syntax` parses templates, `yag-template-analysis` performs semantic checks, `yag-template-envdefs` embeds definitions, and `yag-template-lsp` implements the language server. Utility binaries and scripts are in `cmd/`. Bundled `.ydef` data is in `bundled-defs/`. VS Code extension sources, grammars, and packaging metadata are in `editors/vscode/`. Demo media is in `assets/`.

## Build, Test, and Development Commands

- `cargo build --workspace`: build all Rust crates and binaries.
- `cargo test --workspace`: run Rust unit tests.
- `cargo fmt --all`: format Rust code; use nightly rustfmt because `rustfmt.toml` has unstable options.
- `cargo clippy --workspace --all-targets`: run Rust lints before larger changes.
- `cd editors/vscode && npm install`: install extension tooling.
- `cd editors/vscode && npm run compile`: type-check, lint, and bundle the extension.
- `cd editors/vscode && npm run package`: produce a VSIX package.

For interactive development, use VS Code's `Run Extension` debug configuration.

## Coding Style & Naming Conventions

Rust uses edition 2024 and `rustfmt.toml` with module-level import grouping and `max_width = 120`. Use `snake_case` for modules, functions, and variables; `PascalCase` for types; and keep provider modules grouped by LSP feature under `crates/yag-template-lsp/src/provider/`. TypeScript in `editors/vscode` is formatted by Prettier with tabs, single quotes, trailing commas, and 120-column width. Run `npm run lint` for typed ESLint checks.

## Testing Guidelines

Place Rust unit tests next to the code they exercise with `#[cfg(test)]` modules or focused `#[test]` functions. Prefer parser, analysis, and envdef regression tests for language behavior changes. Run `cargo test --workspace` before submitting Rust changes. Extension tests are not prominent; at minimum run `npm run compile` to validate TypeScript, linting, and bundling.

## Commit & Pull Request Guidelines

Recent history uses concise scoped commits such as `lsp: check for deprecated funcs`, `readme: remove deprecated VS Code badge`, `all: reformat with nightly`, and `release: bump versions to v0.2.6`. Follow the same `scope: imperative summary` pattern.

Pull requests should describe the user-visible effect, mention affected crates or extension files, link related issues, and include screenshots or GIFs for UI-facing VS Code changes. Note the commands run, especially `cargo test --workspace` and `npm run compile`.

## Security & Configuration Tips

Do not commit local VS Code settings, generated `target/`, `dist/`, `out/`, or packaged VSIX files. For LSP debugging, prefer `YAG_LSP_LOG` through `yag-template-lsp.server.extraEnv` rather than hard-coded logging changes.
