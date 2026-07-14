use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use yag_template_envdefs::bundled_envdefs;
use yag_template_format::config::ConfigResolver;
use yag_template_format::{DelimiterPadding, FormatDiagnosticKind, FormatOptions, Indent};

#[derive(Debug, Parser)]
#[command(name = "yagfmt", about = "Format YAG templates")]
struct Args {
    /// Exit with status 1 when formatting would change a file.
    #[arg(long, conflicts_with = "write")]
    check: bool,
    /// Rewrite explicitly named, valid files in place.
    #[arg(long, conflicts_with = "check")]
    write: bool,
    /// Override the maximum line width from project configuration or formatter defaults.
    #[arg(long)]
    width: Option<usize>,
    /// Override block indentation from project configuration or formatter defaults.
    #[arg(long, value_parser = parse_indent)]
    indent: Option<Indent>,
    /// Override continuation indentation from project configuration or formatter defaults.
    #[arg(long, value_parser = parse_indent)]
    continuation_indent: Option<Indent>,
    /// Override ordinary action delimiter padding from project configuration or formatter defaults.
    #[arg(long, value_enum)]
    delimiter_padding: Option<PaddingArg>,
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum PaddingArg {
    None,
    Spaces,
}

impl From<PaddingArg> for DelimiterPadding {
    fn from(value: PaddingArg) -> Self {
        match value {
            PaddingArg::None => Self::None,
            PaddingArg::Spaces => Self::Spaces,
        }
    }
}

fn parse_indent(value: &str) -> Result<Indent, String> {
    if value == "tabs" {
        return Ok(Indent::Tabs);
    }
    let width = value
        .parse::<u8>()
        .map_err(|_| "expected `tabs` or a positive number of spaces".to_owned())?;
    (width > 0)
        .then_some(Indent::Spaces(width))
        .ok_or_else(|| "indentation must be at least one space".to_owned())
}

fn main() {
    std::process::exit(run(Args::parse()));
}

fn run(args: Args) -> i32 {
    if args.write && args.files.is_empty() {
        eprintln!("--write requires one or more explicit file paths; stdin is never written");
        return 2;
    }
    let envdefs = bundled_envdefs::load().expect("bundled envdefs should be valid");

    if args.files.is_empty() {
        let mut source = String::new();
        if let Err(error) = io::stdin().read_to_string(&mut source) {
            eprintln!("failed to read stdin: {error}");
            return 2;
        }
        let mut options = FormatOptions::default();
        apply_cli_overrides(&mut options, &args);
        let result = yag_template_format::format(&source, &envdefs, &options);
        if let Err(error) = io::stdout().write_all(result.text.as_bytes()) {
            eprintln!("failed to write stdout: {error}");
            return 2;
        }
        return if has_parse_error(&result.diagnostics) { 1 } else { 0 };
    }

    let mut resolver = ConfigResolver::default();
    let mut failed = false;
    for path in &args.files {
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("{}: {error}", path.display());
                failed = true;
                continue;
            }
        };
        let mut options = match resolver.resolve_options_for_file(path) {
            Ok(options) => options,
            Err(error) => {
                eprintln!("could not load formatter configuration: {error}");
                return 2;
            }
        };
        apply_cli_overrides(&mut options, &args);
        let result = yag_template_format::format(&source, &envdefs, &options);
        let invalid = has_parse_error(&result.diagnostics);
        failed |= invalid;

        if args.write {
            if !invalid
                && result.text != source
                && let Err(error) = fs::write(path, result.text)
            {
                eprintln!("{}: {error}", path.display());
                failed = true;
            }
        } else if args.check {
            failed |= result.text != source;
        } else if let Err(error) = io::stdout().write_all(result.text.as_bytes()) {
            eprintln!("failed to write stdout: {error}");
            return 2;
        }
    }
    if failed { 1 } else { 0 }
}

fn apply_cli_overrides(options: &mut FormatOptions, args: &Args) {
    if let Some(width) = args.width {
        options.max_width = width;
    }
    if let Some(indent) = args.indent {
        options.indent = indent;
    }
    if let Some(continuation_indent) = args.continuation_indent {
        options.continuation_indent = continuation_indent;
    }
    if let Some(delimiter_padding) = args.delimiter_padding {
        options.delimiter_padding = delimiter_padding.into();
    }
}

fn has_parse_error(diagnostics: &[yag_template_format::FormatDiagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::ParseError)
}
