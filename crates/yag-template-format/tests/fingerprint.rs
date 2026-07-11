mod support;

use support::{assert_formats_preserving_fingerprint, fingerprint};
use yag_template_format::FormatOptions;

#[test]
fn fingerprint_ignores_action_whitespace_and_normal_delimiter_padding() {
    assert_eq!(fingerprint("{{  .Value  }}"), fingerprint("{{.Value}}"));
}

#[test]
fn fingerprint_preserves_pipeline_order() {
    assert_ne!(fingerprint("{{.Value | first | second}}"), fingerprint("{{.Value | second | first}}"));
}

#[test]
fn fingerprint_preserves_assignment_operator() {
    assert_ne!(fingerprint("{{$value := .Value}}"), fingerprint("{{$value = .Value}}"));
}

#[test]
fn fingerprint_preserves_parentheses() {
    assert_ne!(fingerprint("{{(print .Value)}}"), fingerprint("{{print .Value}}"));
}

#[test]
fn fingerprint_preserves_trim_markers() {
    assert_ne!(fingerprint("{{- .Value -}}"), fingerprint("{{ .Value }}"));
}

#[test]
fn fingerprint_preserves_literal_text() {
    assert_ne!(fingerprint("Hello, {{.Value}}!"), fingerprint("Hi, {{.Value}}!"));
}

#[test]
fn fingerprint_preserves_branch_structure() {
    assert_ne!(
        fingerprint("{{if .Value}}{{.Value}}{{else}}{{\"fallback\"}}{{end}}"),
        fingerprint("{{if .Value}}{{.Value}}{{end}}"),
    );
}

#[test]
fn inventory_fixtures_reparse_preserve_fingerprint_and_are_idempotent() {
    for fixture in [
        "inline-prose.tmpl",
        "blocks.tmpl",
        "pipeline.tmpl",
        "comments.tmpl",
        "trim-markers.tmpl",
        "compressed.tmpl",
        "key-value-calls.tmpl",
    ] {
        let source = std::fs::read_to_string(format!("{}/tests/fixtures/inventory/{fixture}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|error| panic!("could not read {fixture}: {error}"));
        assert_formats_preserving_fingerprint(&source, &FormatOptions::default());
    }
}
