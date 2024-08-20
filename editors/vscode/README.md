# YAGPDB Template LSP

A VSCode extension providing rich editor support for YAGPDB's templating language.

**Recognized file extensions:** `.yag`, `.yagcc`, `.gotmpl`, `.go.tmpl`

## Features

Most basic LSP features are supported, notably:

- syntax highlighting
- live error reporting as you type
- code completion for variable and function names
- hover documentation for functions
- inlay hints for function parameter names

in addition to the following niceties:

- variable renaming
- goto definition for variables
- find all references for functions and variables
- automatic indentation
- folding ranges

More sophisticated type-aware code completion is on the roadmap as a long-term goal.
