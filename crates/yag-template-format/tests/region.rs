mod support;

use support::assert_formats_preserving_fingerprint;
use yag_template_format::{FormatDiagnosticKind, FormatOptions, format};

#[test]
fn protected_regions_preserve_literal_boundaries_and_line_counts() {
    for (source, expected) in [
        ("Hello, {{  .User.Username  }}!", "Hello, {{.User.Username}}!"),
        ("- **User:** {{  .User.Username  }}", "- **User:** {{.User.Username}}"),
        ("Use `{{  .Command  }}` here.", "Use `{{.Command}}` here."),
        ("{{  .First  }}{{  .Second  }}", "{{.First}}{{.Second}}"),
        ("   \n  {{  .Value  }}\n\t\n", "   \n  {{.Value}}\n\t\n"),
    ] {
        let result = format(source, &FormatOptions::default());
        assert_eq!(result.text, expected, "unexpected protected-region layout");
        assert_eq!(
            result.text.lines().count(),
            source.lines().count(),
            "line count changed"
        );
        assert_formats_preserving_fingerprint(source, &FormatOptions::default());
    }
}

#[test]
fn region_reports_but_does_not_reflow_protected_textual_overwidth_lines() {
    let source = "Hello, {{ .User.Username }}! This literal line is intentionally too long.";
    let options = FormatOptions {
        max_width: 20,
        ..FormatOptions::default()
    };
    let result = format(source, &options);
    assert_eq!(
        result.text,
        "Hello, {{.User.Username}}! This literal line is intentionally too long."
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::ProtectedOverWidthLine)
    );
    assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn protected_crlf_line_width_excludes_the_line_terminator() {
    let source = "A {{.V}}\r\n";
    let options = FormatOptions {
        max_width: 8,
        ..FormatOptions::default()
    };
    let result = format(source, &options);
    assert_eq!(result.text, source);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.kind != FormatDiagnosticKind::ProtectedOverWidthLine)
    );
}
