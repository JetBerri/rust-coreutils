use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

const GREP: &str = env!("CARGO_BIN_EXE_grep");

fn fixture(name: &str, contents: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("coreutils-grep-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, contents).unwrap();
    path
}

fn run_grep(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(GREP)
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
fn prints_matching_lines_from_file() {
    let a = fixture("match.txt", "apple\nbanana\npineapple\n");

    let output = run_grep(&["apple", path_str(&a)], "");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "apple\npineapple\n"
    );
}

#[test]
fn no_match_exits_with_one() {
    let a = fixture("no_match.txt", "apple\n");

    let output = run_grep(&["cherry", path_str(&a)], "");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
}

#[test]
fn multiple_files_prefix_lines_with_path() {
    let a = fixture("multi_a.txt", "one\nmatch a\n");
    let b = fixture("multi_b.txt", "match b\ntwo\n");

    let output = run_grep(&["match", path_str(&a), path_str(&b)], "");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("{}:match a\n{}:match b\n", path_str(&a), path_str(&b))
    );
}

#[test]
fn missing_file_reports_error_and_continues() {
    let a = fixture("missing_a.txt", "match\n");

    let output = run_grep(&["match", "does-not-exist", path_str(&a)], "");

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("{}:match\n", path_str(&a))
    );
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("grep: does-not-exist: "));
}

#[test]
fn reads_stdin_without_paths() {
    let output = run_grep(&["hola"], "hola\nadios\n");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hola\n");
}

#[test]
fn dash_reads_stdin_between_files() {
    let a = fixture("dash_a.txt", "x a\n");

    let output = run_grep(&["x", path_str(&a), "-"], "x stdin\n");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("{}:x a\n(standard input):x stdin\n", path_str(&a))
    );
}

#[test]
fn last_line_without_newline_gets_one() {
    let output = run_grep(&["end"], "start\nend");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "end\n");
}

#[test]
fn empty_pattern_matches_every_line() {
    let output = run_grep(&[""], "a\nb\n");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "a\nb\n");
}

#[test]
fn non_utf8_input_is_matched_as_bytes() {
    let a = std::env::temp_dir().join(format!("coreutils-grep-bin-{}", std::process::id()));
    fs::write(&a, b"\xff match\n\xfe other\n").unwrap();

    let output = run_grep(&["match", path_str(&a)], "");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"\xff match\n");
}

#[test]
fn closed_stdout_exits_cleanly() {
    let big = fixture("big.txt", &"line\n".repeat(1_000_000));
    let mut child = Command::new(GREP)
        .args(["line", path_str(&big)])
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
