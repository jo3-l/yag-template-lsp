use std::path::{Path, PathBuf};

use itertools::Itertools;

pub(crate) fn ensure_file_contents(path: impl AsRef<Path>, contents: impl AsRef<str>) {
    let path = path.as_ref();
    let contents = normalize_newlines(contents.as_ref());
    if let Ok(old_contents) = std::fs::read_to_string(path) {
        if contents == normalize_newlines(&old_contents) {
            return;
        }
    }

    eprintln!("{} not up-to-date; overwriting...", path.display());
    std::fs::write(path, contents).unwrap();
    panic!("generated file was updated; re-run tests")
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

pub(crate) fn format(tokens: proc_macro2::TokenStream) -> String {
    prettyplease::unparse(&syn::parse2::<syn::File>(tokens).expect("should generate valid code"))
}

pub(crate) fn unwrap_doc(doc: &str) -> String {
    const PARAGRAPH_SEPARATOR: &str = "\n\n";
    doc.split(PARAGRAPH_SEPARATOR)
        .map(|para| textwrap::unfill(para).0)
        .join(PARAGRAPH_SEPARATOR)
}

pub(crate) fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
