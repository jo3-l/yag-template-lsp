use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use yag_template_envdefs::bundled_envdefs;
use yag_template_format::{FormatOptions, format};

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_yagfmt"))
}

fn temp_file(name: &str, source: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("yagfmt-{name}-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join("template.tmpl");
    fs::write(&path, source).unwrap();
    path
}

fn temp_dir(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("yagfmt-{name}-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    directory
}

#[test]
fn formats_stdin_to_stdout() {
    let mut child = command().stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(b"Hello {{ .Name }}").unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"Hello {{ .Name }}\n");
}

#[test]
fn default_options_come_from_the_formatter() {
    let source = "{{if .Enabled}}\n{{dict \"alpha\" \"a long value that forces the formatter to choose multiline layout\" \"beta\" \"another long value that forces the formatter to choose multiline layout\"}}\n{{end}}";
    let mut child = command().stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(source.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    let envdefs = bundled_envdefs::load().unwrap();
    assert_eq!(
        output.stdout,
        format(source, &envdefs, &FormatOptions::default()).text.as_bytes()
    );
}

#[test]
fn check_and_write_are_safe_for_explicit_valid_files() {
    let path = temp_file("check-write", "{{.Name}}");

    assert_eq!(command().arg("--check").arg(&path).status().unwrap().code(), Some(1));
    assert_eq!(fs::read_to_string(&path).unwrap(), "{{.Name}}");
    assert!(command().arg("--write").arg(&path).status().unwrap().success());
    assert_eq!(fs::read_to_string(&path).unwrap(), "{{ .Name }}\n");
    assert!(command().arg("--check").arg(&path).status().unwrap().success());
}

#[test]
fn invalid_input_is_not_written_and_stdin_write_is_rejected() {
    let path = temp_file("invalid", "{{ if");

    assert_eq!(command().arg("--write").arg(&path).status().unwrap().code(), Some(1));
    assert_eq!(fs::read_to_string(&path).unwrap(), "{{ if");
    assert_eq!(command().arg("--write").status().unwrap().code(), Some(2));
    assert_eq!(
        command()
            .args(["--check", "--write", &path.display().to_string()])
            .status()
            .unwrap()
            .code(),
        Some(2)
    );
}

#[test]
fn resolves_config_independently_for_each_file() {
    let root = temp_dir("per-file-config");
    let left_templates = root.join("left/templates");
    let right_templates = root.join("right/templates");
    fs::create_dir_all(&left_templates).unwrap();
    fs::create_dir_all(&right_templates).unwrap();
    fs::write(root.join("left/yagfmt.toml"), "delimiter_padding = \"none\"\n").unwrap();
    fs::write(root.join("right/yagfmt.toml"), "delimiter_padding = \"spaces\"\n").unwrap();
    let left = left_templates.join("template.tmpl");
    let right = right_templates.join("template.tmpl");
    fs::write(&left, "{{ .Left }}").unwrap();
    fs::write(&right, "{{.Right}}").unwrap();

    assert!(
        command()
            .arg("--write")
            .arg(&left)
            .arg(&right)
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(fs::read_to_string(left).unwrap(), "{{.Left}}\n");
    assert_eq!(fs::read_to_string(right).unwrap(), "{{ .Right }}\n");
}

#[test]
fn cli_flags_override_discovered_config() {
    let root = temp_dir("config-cli-override");
    fs::write(root.join("yagfmt.toml"), "delimiter_padding = \"none\"\n").unwrap();
    let path = root.join("template.tmpl");
    fs::write(&path, "{{.Name}}").unwrap();

    assert!(
        command()
            .args(["--write", "--delimiter-padding", "spaces"])
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(fs::read_to_string(path).unwrap(), "{{ .Name }}\n");
}

#[test]
fn invalid_config_stops_processing_with_a_tool_error() {
    let root = temp_dir("invalid-config");
    let invalid_directory = root.join("invalid");
    let later_directory = root.join("later");
    fs::create_dir_all(&invalid_directory).unwrap();
    fs::create_dir_all(&later_directory).unwrap();
    let config_path = invalid_directory.join("yagfmt.toml");
    fs::write(&config_path, "max_wdith = 80\n").unwrap();
    let invalid = invalid_directory.join("template.tmpl");
    let later = later_directory.join("template.tmpl");
    fs::write(&invalid, "{{.Invalid}}").unwrap();
    fs::write(&later, "{{.Later}}").unwrap();

    let output = command().arg("--write").arg(&invalid).arg(&later).output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains(&config_path.display().to_string()));
    assert_eq!(fs::read_to_string(invalid).unwrap(), "{{.Invalid}}");
    assert_eq!(fs::read_to_string(later).unwrap(), "{{.Later}}");
}
