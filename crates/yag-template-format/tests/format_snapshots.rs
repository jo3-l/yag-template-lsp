mod support;

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use support::assert_format_result_preserving_fingerprint;
use yag_template_format::{DelimiterPadding, FormatOptions, Indent, format};

const FIXTURE_ROOT: &str = "tests/snapshots";
const UPDATE_ENV: &str = "YAG_UPDATE_SNAPSHOTS";

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FixtureOptions {
    max_width: Option<usize>,
    indent: Option<FixtureIndent>,
    continuation_indent: Option<FixtureIndent>,
    delimiter_padding: Option<FixtureDelimiterPadding>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FixtureIndent {
    Named(String),
    Spaces(FixtureSpaces),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureSpaces {
    spaces: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FixtureDelimiterPadding {
    None,
    Spaces,
}

#[test]
fn formatter_fixtures_match_raw_snapshots() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_ROOT);
    let fixtures = fixture_files(&root);
    assert!(
        !fixtures.is_empty(),
        "no formatter fixtures found in {}",
        root.display()
    );

    for fixture_path in fixtures {
        let fixture = read_fixture(&fixture_path);
        let result = format(&fixture.source, &fixture.options);
        assert_format_result_preserving_fingerprint(&fixture.source, &fixture.options, &result);
        assert_raw_snapshot(&fixture_path, &result.text);
    }

    assert_no_orphaned_snapshots(&root);
}

struct Fixture {
    source: String,
    options: FormatOptions,
}

fn read_fixture(path: &Path) -> Fixture {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()));
    let (header, source) = split_front_matter(path, &bytes);
    let options = if header.trim().is_empty() {
        FixtureOptions::default()
    } else {
        serde_yaml::from_str::<FixtureOptions>(header)
            .unwrap_or_else(|error| panic!("{}: invalid YAML front matter: {error}", path.display()))
    };
    Fixture {
        source: String::from_utf8(source.to_owned())
            .unwrap_or_else(|error| panic!("{}: template body is not UTF-8: {error}", path.display())),
        options: options.into_format_options(path),
    }
}

fn split_front_matter<'a>(path: &Path, bytes: &'a [u8]) -> (&'a str, &'a [u8]) {
    const OPEN: &[u8] = b"---\n";
    const CLOSE: &[u8] = b"\n---\n";

    if !bytes.starts_with(OPEN) {
        return ("", bytes);
    }
    let rest = &bytes[OPEN.len()..];
    if let Some(source) = rest.strip_prefix(OPEN) {
        return ("", source);
    }
    let close = rest
        .windows(CLOSE.len())
        .position(|window| window == CLOSE)
        .unwrap_or_else(|| panic!("{}: front matter is missing its closing --- delimiter", path.display()));
    let header = std::str::from_utf8(&rest[..close])
        .unwrap_or_else(|error| panic!("{}: front matter is not UTF-8: {error}", path.display()));
    (header, &rest[close + CLOSE.len()..])
}

impl FixtureOptions {
    fn into_format_options(self, path: &Path) -> FormatOptions {
        let mut options = FormatOptions::default();
        if let Some(max_width) = self.max_width {
            assert!(max_width > 0, "{}: max_width must be greater than zero", path.display());
            options.max_width = max_width;
        }
        if let Some(indent) = self.indent {
            options.indent = indent.into_indent(path, "indent");
        }
        if let Some(indent) = self.continuation_indent {
            options.continuation_indent = indent.into_indent(path, "continuation_indent");
        }
        if let Some(delimiter_padding) = self.delimiter_padding {
            options.delimiter_padding = match delimiter_padding {
                FixtureDelimiterPadding::None => DelimiterPadding::None,
                FixtureDelimiterPadding::Spaces => DelimiterPadding::Spaces,
            };
        }
        options
    }
}

impl FixtureIndent {
    fn into_indent(self, path: &Path, option_name: &str) -> Indent {
        match self {
            FixtureIndent::Named(name) if name == "tabs" => Indent::Tabs,
            FixtureIndent::Named(name) => panic!(
                "{}: {option_name} must be `tabs` or {{ spaces: <width> }}, not {name:?}",
                path.display()
            ),
            FixtureIndent::Spaces(FixtureSpaces { spaces }) if spaces > 0 => Indent::Spaces(spaces),
            FixtureIndent::Spaces(_) => panic!("{}: {option_name} spaces must be greater than zero", path.display()),
        }
    }
}

fn assert_raw_snapshot(fixture_path: &Path, actual: &str) {
    let snapshot_path = fixture_path.with_extension("out");
    let update = std::env::var_os(UPDATE_ENV).is_some_and(|value| value == "1");
    match fs::read(&snapshot_path) {
        Ok(expected) if expected == actual.as_bytes() => {}
        Ok(_expected) if update => write_snapshot(&snapshot_path, actual),
        Ok(expected) => panic!(
            "{}: snapshot differs from formatter output\n{}\nexpected:\n{}\nactual:\n{}",
            snapshot_path.display(),
            update_instructions(),
            String::from_utf8_lossy(&expected),
            actual
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && update => write_snapshot(&snapshot_path, actual),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            panic!(
                "{}: snapshot is missing\n{}",
                snapshot_path.display(),
                update_instructions()
            )
        }
        Err(error) => panic!("could not read {}: {error}", snapshot_path.display()),
    }
}

fn write_snapshot(path: &Path, actual: &str) {
    fs::write(path, actual.as_bytes()).unwrap_or_else(|error| panic!("could not write {}: {error}", path.display()));
}

fn update_instructions() -> String {
    format!("run `{UPDATE_ENV}=1 cargo test -p yag-template-format --test format_snapshots` to update it")
}

fn fixture_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files(root, &mut files);
    files.retain(|path| path.extension().is_some_and(|extension| extension == "in"));
    files.sort();
    files
}

fn assert_no_orphaned_snapshots(root: &Path) {
    let mut files = Vec::new();
    collect_files(root, &mut files);
    for snapshot in files
        .into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "out"))
    {
        let fixture = snapshot.with_extension("in");
        assert!(fixture.is_file(), "{}: orphaned snapshot", snapshot.display());
    }
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap_or_else(|error| panic!("{}: {error}", directory.display())) {
        let path = entry
            .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
            .path();
        if path.is_dir() {
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}
