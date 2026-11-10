use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

const CAT: &str = env!("CARGO_BIN_EXE_cat");

fn fixture(name: &str, contents: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("coreutils-cat-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, contents).unwrap();
    path
}

fn run_cat(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(CAT)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn path_str(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn concatenates_files_in_order() {
    let a = fixture("order_a.txt", "a\n");
    let b = fixture("order_b.txt", "b\n");

    let output = run_cat(&[path_str(&a), path_str(&b)], "");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "a\nb\n");
}

#[test]
fn missing_file_reports_error_and_continues() {
    let a = fixture("missing_a.txt", "a\n");
    let b = fixture("missing_b.txt", "b\n");

    let output = run_cat(&[path_str(&a), "does-not-exist", path_str(&b)], "");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "a\nb\n");
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("cat: does-not-exist: "));
}

#[test]
fn reads_stdin_without_paths() {
    let output = run_cat(&[], "hola\n");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hola\n");
}

#[test]
fn dash_reads_stdin_between_files() {
    let a = fixture("dash_a.txt", "a\n");
    let b = fixture("dash_b.txt", "b\n");

    let output = run_cat(&[path_str(&a), "-", path_str(&b)], "c\n");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "a\nc\nb\n");
}

#[test]
fn closed_stdout_exits_cleanly() {
    let big = fixture("big.txt", &"line\n".repeat(1_000_000));
    let mut child = Command::new(CAT)
        .arg(path_str(&big))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut first_bytes = [0u8; 5];
    child
        .stdout
        .take()
        .unwrap()
        .read_exact(&mut first_bytes)
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}
