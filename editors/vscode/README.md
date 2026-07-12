# YAGPDB Template LSP

A VS Code extension providing rich editor support for YAGPDB's templating language.

**Recognized file extensions:** `.yag`, `.yagcc`, `.gotmpl`, `.go.tmpl`

## Features

Most basic LSP features are supported, notably:

- syntax highlighting
- live error reporting as you type
- code completion for variable and function names
- hover documentation for functions
- inlay hints for function parameter names
- document formatting

in addition to the following niceties:

- variable renaming
- goto definition for variables
- find all references for functions and variables
- automatic indentation
- folding ranges

More sophisticated type-aware code completion is on the roadmap as a long-term goal.

## Formatting

Use VS Code's **Format Document** command, or enable `editor.formatOnSave` for YAG files. Formatting uses the closest
`yagfmt.toml` in the file's directory or an ancestor. The configuration file accepts these optional fields:

```toml
max_width = 100
indent = "tabs" # or a positive number of spaces
continuation_indent = "tabs" # or a positive number of spaces
delimiter_padding = "spaces" # or "none"
```
