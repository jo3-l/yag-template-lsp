mod support;

use support::assert_formats_preserving_fingerprint;
use yag_template_format::{FormatOptions, Indent, format};

#[test]
fn indents_if_bodies_and_aligns_else_and_end() {
    let source = "{{ if .Show }}\n  bar baz  \nHello {{ .Name }}\n{{ else }}\n  fallback  \n{{ end }}";
    let expected = "{{if .Show}}\n\tbar baz\n\tHello {{.Name}}\n{{else}}\n\tfallback\n{{end}}";
    let options = FormatOptions::default();

    assert_eq!(format(source, &options).text, expected);
    assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn nested_blocks_use_configured_indentation() {
    let source = "{{if .Outer}}\n{{with .Inner}}\nvalue\n{{else}}\nempty\n{{end}}\n{{end}}";
    let expected = "{{if .Outer}}\n  {{with .Inner}}\n    value\n  {{else}}\n    empty\n  {{end}}\n{{end}}";
    let options = FormatOptions {
        indent: Indent::Spaces(2),
        ..FormatOptions::default()
    };

    assert_eq!(format(source, &options).text, expected);
    assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn all_compound_body_forms_indent_existing_newlines() {
    for (source, expected) in [
        (
            "{{define \"name\"}}\nbody\n{{end}}",
            "{{define \"name\"}}\n\tbody\n{{end}}",
        ),
        (
            "{{block \"name\" .}}\nbody\n{{end}}",
            "{{block \"name\" .}}\n\tbody\n{{end}}",
        ),
        (
            "{{range .Items}}\nitem\n{{else}}\nnone\n{{end}}",
            "{{range .Items}}\n\titem\n{{else}}\n\tnone\n{{end}}",
        ),
        (
            "{{while .Running}}\nwork\n{{else}}\ndone\n{{end}}",
            "{{while .Running}}\n\twork\n{{else}}\n\tdone\n{{end}}",
        ),
        (
            "{{try}}\nwork\n{{catch}}\nrecover\n{{end}}",
            "{{try}}\n\twork\n{{catch}}\n\trecover\n{{end}}",
        ),
    ] {
        let options = FormatOptions::default();
        assert_eq!(format(source, &options).text, expected, "{source}");
        assert_formats_preserving_fingerprint(source, &options);
    }
}

#[test]
fn inline_block_boundaries_and_trim_markers_remain_adjacent() {
    let source = "Hello {{if .Show}}bar{{else}}baz{{end}}\n{{- if .Show -}}\n  trimmed body\n{{- end -}}";
    let expected = "Hello {{if .Show}}bar{{else}}baz{{end}}\n{{- if .Show -}}\n\ttrimmed body\n{{- end -}}";
    let options = FormatOptions::default();

    assert_eq!(format(source, &options).text, expected);
    assert_formats_preserving_fingerprint(source, &options);
}
