mod support;

use support::{fingerprint, has_internal_literal_whitespace_change};

#[test]
fn fingerprint_ignores_action_whitespace_and_normal_delimiter_padding() {
    assert_eq!(fingerprint("{{  .Value  }}"), fingerprint("{{.Value}}"));
}

#[test]
fn fingerprint_preserves_pipeline_order() {
    assert_ne!(
        fingerprint("{{.Value | first | second}}"),
        fingerprint("{{.Value | second | first}}")
    );
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
fn fingerprint_ignores_literal_line_indentation() {
    assert_eq!(
        fingerprint("{{if .Foo}}\nbar baz\n{{end}}"),
        fingerprint("{{if .Foo}}\n\tbar baz  \n{{end}}")
    );
}

#[test]
fn fingerprint_ignores_block_indentation_on_a_protected_display_line() {
    assert_eq!(
        fingerprint("{{if .Foo}}\nHello {{.Name}}\n{{end}}"),
        fingerprint("{{if .Foo}}\n\tHello {{.Name}}\n{{end}}")
    );
}

#[test]
fn fingerprint_preserves_inline_action_text_adjacency() {
    assert_ne!(
        fingerprint("Hello {{if .Foo}}bar{{else}}baz{{end}}"),
        fingerprint("Hello {{if .Foo}} bar{{else}}baz{{end}}")
    );
}

#[test]
fn fingerprint_ignores_whitespace_only_separators_between_flexible_actions() {
    assert_eq!(
        fingerprint("{{$first := 1}} {{$second := 2}}"),
        fingerprint("{{$first := 1}}\n{{$second := 2}}")
    );
    assert_ne!(
        fingerprint("{{.First}} {{.Second}}"),
        fingerprint("{{.First}}\n{{.Second}}")
    );
}

#[test]
fn internal_literal_whitespace_changes_are_detectable_for_a_warning() {
    assert!(has_internal_literal_whitespace_change(
        "{{if .Foo}}\nbar baz\n{{end}}",
        "{{if .Foo}}\nbar  baz\n{{end}}"
    ));
    assert!(!has_internal_literal_whitespace_change(
        "{{if .Foo}}\nbar baz\n{{end}}",
        "{{if .Foo}}\n  bar baz  \n{{end}}"
    ));
}

#[test]
fn fingerprint_preserves_branch_structure() {
    assert_ne!(
        fingerprint("{{if .Value}}{{.Value}}{{else}}{{\"fallback\"}}{{end}}"),
        fingerprint("{{if .Value}}{{.Value}}{{end}}"),
    );
}
