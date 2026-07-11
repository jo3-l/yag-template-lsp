#[allow(dead_code)]
mod support;

use std::path::Path;

use support::fingerprint;
use yag_template_format::{FormatOptions, format};

#[derive(Default)]
struct Metrics {
    total: usize,
    parsed: usize,
    changed: usize,
    rejected: usize,
    diagnostics: usize,
    longest_line: usize,
}

#[test]
#[ignore = "runs the local scratch corpus and prints formatter metrics"]
fn corpus_reparse_fingerprint_and_idempotence_regression() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scratch/corpus");
    let mut metrics = Metrics::default();
    for path in template_files(&root) {
        metrics.total += 1;
        let source = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let parsed = yag_template_syntax::parser::parse(&source);
        if !parsed.errors.is_empty() {
            metrics.rejected += 1;
            continue;
        }
        metrics.parsed += 1;
        let input_fingerprint = fingerprint(&source);
        let result = format(&source, &FormatOptions::default());
        metrics.changed += usize::from(result.text != source);
        metrics.diagnostics += result.diagnostics.len();
        metrics.longest_line = metrics
            .longest_line
            .max(result.text.lines().map(str::len).max().unwrap_or_default());

        let output = yag_template_syntax::parser::parse(&result.text);
        assert!(
            output.errors.is_empty(),
            "{}: formatted output did not parse: {:?}",
            path.display(),
            output.errors
        );
        assert_eq!(
            fingerprint(&result.text),
            input_fingerprint,
            "{}: fingerprint changed",
            path.display()
        );
        assert_eq!(
            format(&result.text, &FormatOptions::default()).text,
            result.text,
            "{}: not idempotent",
            path.display()
        );
    }
    println!(
        "corpus: total={} parsed={} changed={} rejected={} diagnostics={} longest_line={}",
        metrics.total, metrics.parsed, metrics.changed, metrics.rejected, metrics.diagnostics, metrics.longest_line
    );
}

fn template_files(directory: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(directory).unwrap_or_else(|error| panic!("{}: {error}", directory.display())) {
        let path = entry
            .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
            .path();
        if path.is_dir() {
            files.extend(template_files(&path));
        } else if path.extension().is_some_and(|extension| extension == "tmpl") {
            files.push(path);
        }
    }
    files.sort();
    files
}
