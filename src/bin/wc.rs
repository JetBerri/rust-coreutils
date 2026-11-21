use std::{
    fs::File,
    io::{self, Read, Write},
    ops::AddAssign,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;

const STDIN_PATH: &str = "-";
const TOTAL_LABEL: &str = "total";
const BUFFER_SIZE: usize = 64 * 1024;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    paths: Vec<PathBuf>,
}

#[derive(Default, Clone, Copy)]
struct Counts {
    lines: usize,
    words: usize,
    bytes: usize,
}

impl AddAssign for Counts {
    fn add_assign(&mut self, other: Self) {
        self.lines += other.lines;
        self.words += other.words;
        self.bytes += other.bytes;
    }
}

fn main() -> ExitCode {
    let args = Cli::parse();
    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(STDIN_PATH)]
    } else {
        args.paths
    };

    let mut out = io::stdout().lock();
    let mut total = Counts::default();
    let mut failed = false;

    for path in &paths {
        let result = wc_path(path).and_then(|counts| {
            total += counts;
            write_counts(&counts, display_name(path).as_deref(), &mut out)
        });
        match result {
            Ok(()) => {}
            Err(e) if is_broken_pipe(&e) => return ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("wc: {}: {}", path.display(), e);
                failed = true;
            }
        }
    }

    let result = if paths.len() > 1 {
        write_counts(&total, Some(TOTAL_LABEL), &mut out).and_then(|()| out.flush())
    } else {
        out.flush()
    };

    match result {
        Err(e) if !is_broken_pipe(&e) => {
            eprintln!("wc: write error: {}", e);
            ExitCode::FAILURE
        }
        _ if failed => ExitCode::FAILURE,
        _ => ExitCode::SUCCESS,
    }
}

fn display_name(path: &Path) -> Option<String> {
    if path == Path::new(STDIN_PATH) {
        return None;
    }
    Some(path.display().to_string())
}

fn wc_path(path: &Path) -> io::Result<Counts> {
    if path == Path::new(STDIN_PATH) {
        return wc(io::stdin().lock());
    }
    wc(File::open(path)?)
}

fn wc(mut input: impl Read) -> io::Result<Counts> {
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut counts = Counts::default();
    let mut in_word = false;

    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            return Ok(counts);
        }
        for &byte in &buffer[..read] {
            if byte == b'\n' {
                counts.lines += 1;
            }
            let is_space = byte.is_ascii_whitespace();
            if !is_space && !in_word {
                counts.words += 1;
            }
            in_word = !is_space;
        }
        counts.bytes += read;
    }
}

fn write_counts(counts: &Counts, name: Option<&str>, out: &mut impl Write) -> io::Result<()> {
    write!(
        out,
        "{:>8}{:>8}{:>8}",
        counts.lines, counts.words, counts.bytes
    )?;
    match name {
        Some(name) => writeln!(out, " {name}"),
        None => writeln!(out),
    }
}

fn is_broken_pipe(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::BrokenPipe
}
