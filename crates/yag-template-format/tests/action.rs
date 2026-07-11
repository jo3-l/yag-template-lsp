mod support;

use support::assert_formats_preserving_fingerprint;
use yag_template_format::{DelimiterPadding, FormatDiagnosticKind, FormatOptions, format};

#[test]
fn formats_ordinary_actions_and_clauses_with_no_delimiter_padding() {
    let source = "{{  $value := .User.Name  }}\n{{ if  .Enabled }}{{ template \"card\" . }}{{ else if .Admin }}{{return .User}}{{else}}{{end}}";
    let expected = "{{$value := .User.Name}}\n{{if .Enabled}}{{template \"card\" .}}{{else if .Admin}}{{return .User}}{{else}}{{end}}";
    let options = FormatOptions::default();

    assert_eq!(format(source, &options).text, expected);
    assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn spaces_padding_applies_to_ordinary_and_protected_actions_without_reflowing_text() {
    let source = "Hello, {{.User.Name}}!\n{{$value := .User.Name}}";
    let expected = "Hello, {{ .User.Name }}!\n{{ $value := .User.Name }}";
    let options = FormatOptions {
        delimiter_padding: DelimiterPadding::Spaces,
        ..FormatOptions::default()
    };

    assert_eq!(format(source, &options).text, expected);
    assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn comments_trim_actions_and_multiline_actions_remain_verbatim() {
    let source = "{{/* comment */}}\n{{- $value := .User -}}\n{{\n  .User\n}}";
    let spaces = FormatOptions {
        delimiter_padding: DelimiterPadding::Spaces,
        ..FormatOptions::default()
    };

    assert_eq!(format(source, &FormatOptions::default()).text, source);
    assert_eq!(format(source, &spaces).text, source);
    assert_formats_preserving_fingerprint(source, &spaces);
}

#[test]
fn padding_is_idempotent_for_both_modes() {
    for options in [
        FormatOptions::default(),
        FormatOptions {
            delimiter_padding: DelimiterPadding::Spaces,
            ..FormatOptions::default()
        },
    ] {
        assert_formats_preserving_fingerprint("{{  .Value  }} {{ $x := 1 }}", &options);
    }
}

#[test]
fn protected_width_diagnostics_measure_the_formatted_action() {
    let options = FormatOptions {
        delimiter_padding: DelimiterPadding::Spaces,
        max_width: 8,
        ..FormatOptions::default()
    };
    let result = format("A {{.V}}", &options);

    assert_eq!(result.text, "A {{ .V }}");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::ProtectedOverWidthLine)
    );
}
