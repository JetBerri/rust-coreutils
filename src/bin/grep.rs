use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;

const STDIN_PATH: &str = "-";
const STDIN_LABEL: &str = "(standard input)";
const EXIT_NO_MATCH: u8 = 1;
const EXIT_ERROR: u8 = 2;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    pattern: String,
    paths: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let args = Cli::parse();
    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(STDIN_PATH)]
    } else {
        args.paths
    };
    let show_path = paths.len() > 1;
    let pattern = args.pattern.as_bytes();

    let mut out = io::stdout().lock();
    let mut matched = false;
    let mut failed = false;

    for path in &paths {
        let prefix = show_path.then(|| line_prefix(path));
        match grep_path(path, pattern, prefix.as_deref(), &mut out) {
            Ok(found) => matched |= found,
            Err(e) if is_broken_pipe(&e) => return ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("grep: {}: {}", path.display(), e);
                failed = true;
            }
        }
    }

    match out.flush() {
        Err(e) if !is_broken_pipe(&e) => {
            eprintln!("grep: write error: {}", e);
            ExitCode::from(EXIT_ERROR)
        }
        _ if failed => ExitCode::from(EXIT_ERROR),
        _ if matched => ExitCode::SUCCESS,
        _ => ExitCode::from(EXIT_NO_MATCH),
    }
}

fn line_prefix(path: &Path) -> String {
    if path == Path::new(STDIN_PATH) {
        return format!("{STDIN_LABEL}:");
    }
    format!("{}:", path.display())
}

fn grep_path(
    path: &Path,
    pattern: &[u8],
    prefix: Option<&str>,
    out: &mut impl Write,
) -> io::Result<bool> {
    if path == Path::new(STDIN_PATH) {
        return grep(io::stdin().lock(), pattern, prefix, out);
    }
    grep(BufReader::new(File::open(path)?), pattern, prefix, out)
}

fn grep(
    mut input: impl BufRead,
    pattern: &[u8],
    prefix: Option<&str>,
    out: &mut impl Write,
) -> io::Result<bool> {
    let mut line = Vec::new();
    let mut matched = false;

    while input.read_until(b'\n', &mut line)? > 0 {
        if contains(&line, pattern) {
            write_line(&line, prefix, out)?;
            matched = true;
        }
        line.clear();
    }
    Ok(matched)
}

fn contains(line: &[u8], pattern: &[u8]) -> bool {
    pattern.is_empty() || line.windows(pattern.len()).any(|window| window == pattern)
}

fn write_line(line: &[u8], prefix: Option<&str>, out: &mut impl Write) -> io::Result<()> {
    if let Some(prefix) = prefix {
        out.write_all(prefix.as_bytes())?;
    }
    out.write_all(line)?;
    if !line.ends_with(b"\n") {
        out.write_all(b"\n")?;
    }
    Ok(())
}

fn is_broken_pipe(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::BrokenPipe
}
