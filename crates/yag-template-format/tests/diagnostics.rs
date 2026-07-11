mod support;

use yag_template_format::{DelimiterPadding, FormatDiagnosticKind, FormatOptions, format};

#[test]
fn protected_width_diagnostics_measure_the_formatted_action() {
    let options = FormatOptions {
        delimiter_padding: DelimiterPadding::Spaces,
        max_width: 8,
        ..FormatOptions::default()
    };
    let source = "A {{.V}}";
    let result = format(source, &options);

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::ProtectedOverWidthLine)
    );
    support::assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn protected_width_diagnostics_survive_earlier_flexible_wrapping() {
    let options = FormatOptions {
        max_width: 12,
        ..FormatOptions::default()
    };
    let source = "{{print .A .B .C}}\nHello {{ .Very.Long.Field }}!";
    let result = format(source, &options);

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::ProtectedOverWidthLine)
    );
    support::assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn odd_key_value_arguments_report_a_diagnostic() {
    let options = FormatOptions {
        max_width: 14,
        ..FormatOptions::default()
    };
    let source = "{{sdict \"a\" \"one\" \"dangling\"}}";
    let result = format(source, &options);

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::OddKeyValueArgumentCount)
    );
    support::assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn protected_textual_overwidth_is_reported_without_reflowing() {
    let source = "Hello, {{ .User.Username }}! This literal line is intentionally too long.";
    let options = FormatOptions {
        max_width: 20,
        ..FormatOptions::default()
    };
    let result = format(source, &options);

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::ProtectedOverWidthLine)
    );
    support::assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn protected_crlf_line_width_excludes_the_line_terminator() {
    let source = "A {{.V}}\r\n";
    let options = FormatOptions {
        max_width: 8,
        ..FormatOptions::default()
    };
    let result = format(source, &options);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.kind != FormatDiagnosticKind::ProtectedOverWidthLine)
    );
    support::assert_formats_preserving_fingerprint(source, &options);
}
