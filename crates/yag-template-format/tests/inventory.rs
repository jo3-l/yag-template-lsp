const INVENTORY: &[&str] = &[
    "inline-prose.tmpl",
    "blocks.tmpl",
    "pipeline.tmpl",
    "comments.tmpl",
    "trim-markers.tmpl",
    "compressed.tmpl",
    "key-value-calls.tmpl",
];

#[test]
fn inventory_examples_parse() {
    for fixture in INVENTORY {
        let source = std::fs::read_to_string(format!(
            "{}/tests/fixtures/inventory/{fixture}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap_or_else(|error| panic!("could not read {fixture}: {error}"));
        let parsed = yag_template_syntax::parser::parse(&source);
        assert!(parsed.errors.is_empty(), "{fixture} did not parse: {:?}", parsed.errors);
    }
}
