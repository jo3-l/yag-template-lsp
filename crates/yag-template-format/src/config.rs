//! Resolve formatter options from project configuration files.

use std::collections::HashMap;
use std::error::Error;
use std::num::{NonZeroU8, NonZeroUsize};
use std::path::{Path, PathBuf};
use std::{fmt, fs};

use serde::Deserialize;

use crate::{DelimiterPadding, FormatOptions, Indent};

/// The filename searched for in template directories and their ancestors.
pub const CONFIG_FILE_NAME: &str = "yagfmt.toml";

/// An error encountered while locating or interpreting a formatter configuration file.
#[derive(Debug)]
pub struct ConfigError {
    path: PathBuf,
    message: String,
}

impl ConfigError {
    fn new(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }

    /// The input or configuration path that caused this error.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.message)
    }
}

impl Error for ConfigError {}

/// Resolves formatter options while caching discoveries made during one run.
///
/// Each searched directory is cached as either using its nearest configuration
/// file or having no configuration. Parsed configuration files are cached by
/// path, so sibling files and nested directories do not repeatedly read and
/// parse the same file. A resolver intentionally does not observe filesystem
/// changes after it has cached a result.
#[derive(Default)]
pub struct ConfigResolver {
    config_by_directory: HashMap<PathBuf, Option<PathBuf>>,
    options_by_config: HashMap<PathBuf, FormatOptions>,
}

impl ConfigResolver {
    /// Resolve options for a template file using this resolver's cache.
    ///
    /// The nearest [`CONFIG_FILE_NAME`] in the file's parent directory or an
    /// ancestor is applied over [`FormatOptions::default`]. Missing
    /// configuration files are ignored; a discovered file that cannot be read
    /// or validated is an error. The input path need not exist, which permits
    /// editor buffers to use their intended on-disk location.
    pub fn resolve_options_for_file(&mut self, path: &Path) -> Result<FormatOptions, ConfigError> {
        let mut directory = input_directory(path)?;
        let mut searched_directories = Vec::new();

        loop {
            if let Some(config_path) = self.config_by_directory.get(&directory) {
                let config_path = config_path.clone();
                self.cache_directories(&searched_directories, config_path.clone());
                return match config_path {
                    Some(config_path) => Ok(self
                        .options_by_config
                        .get(&config_path)
                        .expect("cached configuration path must have parsed options")
                        .clone()),
                    None => Ok(FormatOptions::default()),
                };
            }

            searched_directories.push(directory.clone());
            let config_path = directory.join(CONFIG_FILE_NAME);
            match fs::read_to_string(&config_path) {
                Ok(source) => {
                    let options = parse_options(&config_path, &source)?;
                    self.options_by_config.insert(config_path.clone(), options.clone());
                    self.cache_directories(&searched_directories, Some(config_path));
                    return Ok(options);
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(ConfigError::new(config_path, error.to_string())),
            }

            if !directory.pop() {
                self.cache_directories(&searched_directories, None);
                return Ok(FormatOptions::default());
            }
        }
    }

    fn cache_directories(&mut self, directories: &[PathBuf], config_path: Option<PathBuf>) {
        for directory in directories {
            self.config_by_directory.insert(directory.clone(), config_path.clone());
        }
    }
}

/// Resolve options for a template file.
///
/// The nearest [`CONFIG_FILE_NAME`] in the file's parent directory or an
/// ancestor is applied over [`FormatOptions::default`]. Missing configuration
/// files are ignored; a discovered file that cannot be read or validated is an
/// error. The input path need not exist, which permits editor buffers to use
/// their intended on-disk location.
pub fn resolve_options_for_file(path: &Path) -> Result<FormatOptions, ConfigError> {
    ConfigResolver::default().resolve_options_for_file(path)
}

fn input_directory(path: &Path) -> Result<PathBuf, ConfigError> {
    let absolute_path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|error| ConfigError::new(path, error.to_string()))?
            .join(path)
    };
    absolute_path
        .parent()
        .map(Path::to_owned)
        .ok_or_else(|| ConfigError::new(path, "input path has no parent directory"))
}

fn parse_options(path: &Path, source: &str) -> Result<FormatOptions, ConfigError> {
    let file_options =
        toml::from_str::<FileOptions>(source).map_err(|error| ConfigError::new(path, error.to_string()))?;
    file_options.apply(path)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileOptions {
    max_width: Option<NonZeroUsize>,
    indent: Option<FileIndent>,
    continuation_indent: Option<FileIndent>,
    delimiter_padding: Option<FileDelimiterPadding>,
}

impl FileOptions {
    fn apply(self, path: &Path) -> Result<FormatOptions, ConfigError> {
        let mut options = FormatOptions::default();
        if let Some(max_width) = self.max_width {
            options.max_width = max_width.get();
        }
        if let Some(indent) = self.indent {
            options.indent = indent.into_indent(path, "indent")?;
        }
        if let Some(continuation_indent) = self.continuation_indent {
            options.continuation_indent = continuation_indent.into_indent(path, "continuation_indent")?;
        }
        if let Some(delimiter_padding) = self.delimiter_padding {
            options.delimiter_padding = delimiter_padding.into();
        }
        Ok(options)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FileIndent {
    Tabs(String),
    Spaces(NonZeroU8),
}

impl FileIndent {
    fn into_indent(self, path: &Path, field: &str) -> Result<Indent, ConfigError> {
        match self {
            FileIndent::Tabs(value) if value == "tabs" => Ok(Indent::Tabs),
            FileIndent::Tabs(value) => Err(ConfigError::new(
                path,
                format!("`{field}` must be `tabs` or a positive number of spaces, got `{value}`"),
            )),
            FileIndent::Spaces(width) => Ok(Indent::Spaces(width.get())),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FileDelimiterPadding {
    None,
    Spaces,
}

impl From<FileDelimiterPadding> for DelimiterPadding {
    fn from(value: FileDelimiterPadding) -> Self {
        match value {
            FileDelimiterPadding::None => Self::None,
            FileDelimiterPadding::Spaces => Self::Spaces,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{ConfigResolver, resolve_options_for_file};
    use crate::{DelimiterPadding, FormatOptions, Indent};

    static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let unique = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("yag-template-format-{name}-{}-{unique}", std::process::id()));
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
    fn nearest_config_overlays_defaults_without_merging_ancestors() {
        let root = TempDir::new("nearest-config");
        fs::write(
            root.path().join("yagfmt.toml"),
            "max_width = 40\ndelimiter_padding = \"none\"\n",
        )
        .unwrap();
        let project = root.path().join("project");
        let nested = project.join("templates");
        fs::create_dir_all(&nested).unwrap();
        fs::write(project.join("yagfmt.toml"), "indent = 2\n").unwrap();

        let options = resolve_options_for_file(&nested.join("template.gotmpl")).unwrap();

        assert_eq!(
            options,
            FormatOptions {
                indent: Indent::Spaces(2),
                ..FormatOptions::default()
            }
        );
    }

    #[test]
    fn config_applies_all_supported_visual_options() {
        let root = TempDir::new("visual-options");
        fs::write(
            root.path().join("yagfmt.toml"),
            "max_width = 80\nindent = \"tabs\"\ncontinuation_indent = 4\ndelimiter_padding = \"none\"\n",
        )
        .unwrap();

        let options = resolve_options_for_file(&root.path().join("template.gotmpl")).unwrap();

        assert_eq!(options.max_width, 80);
        assert_eq!(options.indent, Indent::Tabs);
        assert_eq!(options.continuation_indent, Indent::Spaces(4));
        assert_eq!(options.delimiter_padding, DelimiterPadding::None);
    }

    #[test]
    fn missing_config_uses_formatter_defaults() {
        let root = TempDir::new("missing-config");

        assert_eq!(
            resolve_options_for_file(&root.path().join("template.gotmpl")).unwrap(),
            FormatOptions::default()
        );
    }

    #[test]
    fn config_rejects_unknown_and_invalid_values() {
        let root = TempDir::new("invalid-config");
        let config_path = root.path().join("yagfmt.toml");
        fs::write(&config_path, "max_wdith = 80\n").unwrap();
        let unknown_error = resolve_options_for_file(&root.path().join("template.gotmpl")).unwrap_err();
        assert_eq!(unknown_error.path(), config_path);
        assert!(unknown_error.to_string().contains("unknown field"));

        fs::write(&config_path, "continuation_indent = 0\n").unwrap();
        let invalid_error = resolve_options_for_file(&root.path().join("template.gotmpl")).unwrap_err();
        assert_eq!(invalid_error.path(), config_path);
    }

    #[test]
    fn resolver_caches_discovered_configs_across_directories() {
        let root = TempDir::new("cached-config");
        let left = root.path().join("left");
        let right = root.path().join("right");
        fs::create_dir_all(&left).unwrap();
        fs::create_dir_all(&right).unwrap();
        let config_path = root.path().join("yagfmt.toml");
        fs::write(&config_path, "delimiter_padding = \"none\"\n").unwrap();

        let mut resolver = ConfigResolver::default();
        assert_eq!(
            resolver
                .resolve_options_for_file(&left.join("template.gotmpl"))
                .unwrap()
                .delimiter_padding,
            DelimiterPadding::None
        );

        fs::write(&config_path, "delimiter_padding = \"spaces\"\n").unwrap();
        assert_eq!(
            resolver
                .resolve_options_for_file(&right.join("template.gotmpl"))
                .unwrap()
                .delimiter_padding,
            DelimiterPadding::None
        );
    }

    #[test]
    fn resolver_caches_missing_configs_by_directory() {
        let root = TempDir::new("cached-missing-config");
        let templates = root.path().join("templates");
        fs::create_dir_all(&templates).unwrap();

        let mut resolver = ConfigResolver::default();
        assert_eq!(
            resolver
                .resolve_options_for_file(&templates.join("first.gotmpl"))
                .unwrap(),
            FormatOptions::default()
        );

        fs::write(templates.join("yagfmt.toml"), "delimiter_padding = \"none\"\n").unwrap();
        assert_eq!(
            resolver
                .resolve_options_for_file(&templates.join("second.gotmpl"))
                .unwrap(),
            FormatOptions::default()
        );
    }
}
