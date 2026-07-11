mod support;

use support::assert_formats_preserving_fingerprint;
use yag_template_format::{FormatDiagnosticKind, FormatOptions, Indent, format};

#[test]
fn expressions_and_pipelines_stay_flat_at_the_fit_boundary() {
    let options = FormatOptions {
        max_width: 28,
        ..FormatOptions::default()
    };
    let source = "{{print .A .B .C}}\n{{.Value | first | second}}";

    assert_eq!(format(source, &options).text, source);
    assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn expressions_and_pipelines_break_with_the_configured_continuation_indent() {
    let options = FormatOptions {
        max_width: 12,
        continuation_indent: Indent::Spaces(2),
        ..FormatOptions::default()
    };
    let source = "{{print .A .B .C}}\n{{.Value | first | second}}";
    let expected = "{{print\n  .A\n  .B\n  .C}}\n{{.Value\n  | first\n  | second}}";

    assert_eq!(format(source, &options).text, expected);
    assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn nested_calls_parentheses_assignments_and_headers_use_document_layout() {
    let options = FormatOptions {
        max_width: 14,
        continuation_indent: Indent::Spaces(2),
        ..FormatOptions::default()
    };
    let source = "{{$value := print (printf \"%s\" .Name)}}\n{{if print .A .B .C}}\nbody\n{{end}}";
    let expected = "{{$value :=\n  print\n    (printf\n      \"%s\"\n      .Name)}}\n{{if\n  print\n    .A\n    .B\n    .C}}\n\tbody\n{{end}}";

    assert_eq!(format(source, &options).text, expected);
    assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn protected_display_actions_remain_flat_when_they_exceed_width() {
    let options = FormatOptions {
        max_width: 4,
        ..FormatOptions::default()
    };
    let source = "Hello {{ .Very.Long.Field }}!";
    let expected = "Hello {{.Very.Long.Field}}!";

    assert_eq!(format(source, &options).text, expected);
    assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn protected_width_diagnostics_survive_earlier_flexible_wrapping() {
    let options = FormatOptions {
        max_width: 12,
        ..FormatOptions::default()
    };
    let result = format("{{print .A .B .C}}\nHello {{ .Very.Long.Field }}!", &options);

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::ProtectedOverWidthLine)
    );
}
