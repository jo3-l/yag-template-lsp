use tower_lsp::lsp_types::{DocumentFormattingParams, MessageType, TextEdit, Url};
use yag_template_format::config::{ConfigError, resolve_options_for_file};
use yag_template_format::{FormatDiagnosticKind, FormatOptions, format};

use crate::session::Session;

pub(crate) async fn format_document(
    sess: &Session,
    params: DocumentFormattingParams,
) -> anyhow::Result<Option<Vec<TextEdit>>> {
    let uri = params.text_document.uri;
    let options = match options_for_uri(&uri) {
        Ok(options) => options,
        Err(error) => {
            sess.client
                .show_message(
                    MessageType::ERROR,
                    format!("Could not load formatter configuration: {error}"),
                )
                .await;
            return Ok(None);
        }
    };
    let doc = sess.document(&uri)?;
    let Some(text) = format_with_options(&doc.source, &options) else {
        return Ok(None);
    };

    let range = doc.mapper.range(doc.syntax().text_range());
    Ok(Some(vec![TextEdit::new(range, text)]))
}

fn options_for_uri(uri: &Url) -> Result<FormatOptions, ConfigError> {
    match uri.to_file_path() {
        Ok(path) => resolve_options_for_file(&path),
        Err(()) => Ok(FormatOptions::default()),
    }
}

fn format_with_options(source: &str, options: &FormatOptions) -> Option<String> {
    let result = format(source, options);
    if result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::ParseError)
        || result.text == source
    {
        None
    } else {
        Some(result.text)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tower_lsp::lsp_types::Url;

    use super::{format_with_options, options_for_uri};

    static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let unique = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("yag-template-lsp-{name}-{}-{unique}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn formatting_uses_options_resolved_for_the_document_path() {
        let root = TempDir::new("formatting-options");
        fs::write(root.path().join("yagfmt.toml"), "delimiter_padding = \"none\"\n").unwrap();
        let uri = Url::from_file_path(root.path().join("template.gotmpl")).unwrap();
        let options = options_for_uri(&uri).unwrap();

        assert_eq!(
            format_with_options("{{ .Name }}", &options),
            Some("{{.Name}}\n".to_owned())
        );
    }

    #[test]
    fn invalid_templates_do_not_produce_edits() {
        assert_eq!(format_with_options("{{ if", &Default::default()), None);
    }

    #[test]
    fn invalid_config_is_reported_for_a_file_uri() {
        let root = TempDir::new("invalid-config");
        fs::write(root.path().join("yagfmt.toml"), "indent = 0\n").unwrap();
        let uri = Url::from_file_path(root.path().join("template.gotmpl")).unwrap();

        assert!(options_for_uri(&uri).is_err());
    }
}
