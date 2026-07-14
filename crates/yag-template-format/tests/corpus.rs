mod support;

use std::path::{Path, PathBuf};

use support::{assert_format_result_preserving_fingerprint, bundled_envdefs};
use yag_template_format::{FormatOptions, format};

const CORPUS_ROOT: &str = "tests/corpus/yagpdb-cc/src";

#[test]
fn yagpdb_cc_corpus_respects_formatter_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS_ROOT);
    let templates = template_files(&root);
    assert!(!templates.is_empty(), "no templates found in {}", root.display());
    let envdefs = bundled_envdefs();

    for path in templates {
        let source = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let result = format(&source, &envdefs, &FormatOptions::default());
        assert_format_result_preserving_fingerprint(&source, &FormatOptions::default(), &result, path.display());
    }
}

fn template_files(directory: &Path) -> Vec<PathBuf> {
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
