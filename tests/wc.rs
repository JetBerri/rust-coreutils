use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

const WC: &str = env!("CARGO_BIN_EXE_wc");

fn fixture(name: &str, contents: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("coreutils-wc-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, contents).unwrap();
    path
}

fn run_wc(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(WC)
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
fn counts_lines_words_and_bytes_of_file() {
    let a = fixture("single.txt", "hello world\nfoo\n");

    let output = run_wc(&[path_str(&a)], "");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("       2       3      16 {}\n", path_str(&a))
    );
}

#[test]
fn reads_stdin_without_paths_and_prints_no_name() {
    let output = run_wc(&[], "one two three\n");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "       1       3      14\n"
    );
}

#[test]
fn multiple_files_print_total() {
    let a = fixture("total_a.txt", "a\n");
    let b = fixture("total_b.txt", "b c\n");

    let output = run_wc(&[path_str(&a), path_str(&b)], "");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "       1       1       2 {}\n       1       2       4 {}\n       2       3       6 total\n",
            path_str(&a),
            path_str(&b)
        )
    );
}

#[test]
fn missing_file_reports_error_and_continues() {
    let a = fixture("missing_a.txt", "a\n");

    let output = run_wc(&["does-not-exist", path_str(&a)], "");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "       1       1       2 {}\n       1       1       2 total\n",
            path_str(&a)
        )
    );
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("wc: does-not-exist: "));
}

#[test]
fn counts_consecutive_whitespace_as_one_separator() {
    let output = run_wc(&[], "  a \t b\n\n c  ");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "       2       3      13\n"
    );
}

#[test]
fn last_line_without_newline_is_not_counted_as_line() {
    let output = run_wc(&[], "a\nb");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "       1       2       3\n"
    );
}

#[test]
fn empty_input_counts_zero() {
    let output = run_wc(&[], "");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "       0       0       0\n"
    );
}
