mod support;

use std::collections::BTreeMap;

use support::assert_formats_preserving_fingerprint;
use yag_template_format::{
    DanglingValuePolicy, FormatDiagnosticKind, FormatOptions, FunctionLayouts, LayoutKind, format,
};

#[test]
fn default_dict_layouts_keep_pairs_together_when_broken() {
    let options = FormatOptions {
        max_width: 20,
        ..FormatOptions::default()
    };
    let source = "{{(sdict \"a\" \"one\" \"b\" \"two\")}}\n{{dict \"first\" .User.First \"last\" .User.Last}}";
    let expected =
        "{{(sdict\n  \"a\" \"one\"\n  \"b\" \"two\")}}\n{{dict\n  \"first\" .User.First\n  \"last\" .User.Last}}";

    assert_eq!(format(source, &options).text, expected);
    assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn configured_key_value_functions_dispatch_by_exact_name() {
    let options = FormatOptions {
        max_width: 18,
        function_layouts: FunctionLayouts {
            by_name: BTreeMap::from([(
                "metadata".to_owned(),
                LayoutKind::KeyValuePairs {
                    dangling_value: DanglingValuePolicy::PreserveCallLayout,
                },
            )]),
        },
        ..FormatOptions::default()
    };
    let source = "{{metadata \"name\" (print .First .Last) \"active\" true}}";
    let expected = "{{metadata\n  \"name\" (print\n    .First\n    .Last)\n  \"active\" true}}";

    assert_eq!(format(source, &options).text, expected);
    assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn odd_key_value_arguments_fall_back_to_generic_call_layout_with_a_diagnostic() {
    let options = FormatOptions {
        max_width: 14,
        ..FormatOptions::default()
    };
    let source = "{{sdict \"a\" \"one\" \"dangling\"}}";
    let expected = "{{sdict\n  \"a\"\n  \"one\"\n  \"dangling\"}}";
    let result = format(source, &options);

    assert_eq!(result.text, expected);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::OddKeyValueArgumentCount)
    );
    assert_formats_preserving_fingerprint(source, &options);
}

#[test]
fn key_value_pairs_stay_flat_at_the_fit_boundary() {
    let options = FormatOptions {
        max_width: 28,
        ..FormatOptions::default()
    };
    let source = "{{dict \"a\" \"one\" \"b\" \"two\"}}";

    assert_eq!(format(source, &options).text, source);
    assert_formats_preserving_fingerprint(source, &options);
}
