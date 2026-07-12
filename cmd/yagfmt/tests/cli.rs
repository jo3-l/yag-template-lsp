use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

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

#[test]
fn formats_stdin_to_stdout() {
    let mut child = command()
        .args(["--stdin-filepath", "editor-buffer.tmpl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"Hello {{ .Name }}").unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"Hello {{ .Name }}");
}

#[test]
fn default_options_come_from_the_formatter() {
    let source = "{{if .Enabled}}\n{{dict \"alpha\" \"a long value that forces the formatter to choose multiline layout\" \"beta\" \"another long value that forces the formatter to choose multiline layout\"}}\n{{end}}";
    let mut child = command().stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(source.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, format(source, &FormatOptions::default()).text.as_bytes());
}

#[test]
fn check_and_write_are_safe_for_explicit_valid_files() {
    let path = temp_file("check-write", "{{.Name}}");

    assert_eq!(command().arg("--check").arg(&path).status().unwrap().code(), Some(1));
    assert_eq!(fs::read_to_string(&path).unwrap(), "{{.Name}}");
    assert!(command().arg("--write").arg(&path).status().unwrap().success());
    assert_eq!(fs::read_to_string(&path).unwrap(), "{{ .Name }}");
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
fn layout_flags_apply_to_stdin() {
    let mut child = command()
        .args([
            "--width",
            "20",
            "--delimiter-padding",
            "spaces",
            "--continuation-indent",
            "2",
            "--key-value-function",
            "metadata",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"{{metadata \"a\" \"one\" \"b\" \"two\"}}")
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"{{ metadata\n  \"a\" \"one\"\n  \"b\" \"two\" }}");
}
