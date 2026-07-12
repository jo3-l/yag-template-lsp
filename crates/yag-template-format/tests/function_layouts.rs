mod support;

use std::collections::BTreeMap;

use yag_template_format::{DanglingValuePolicy, FormatOptions, FunctionLayouts, LayoutKind, format};

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
    let expected = "{{ metadata\n\t\"name\" (print\n\t\t.First\n\t\t.Last)\n\t\"active\" true }}";

    assert_eq!(format(source, &options).text, expected);
    support::assert_formats_preserving_fingerprint(source, &options);
}
