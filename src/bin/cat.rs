use std::{
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;

const STDIN_PATH: &str = "-";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    paths: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let args = Cli::parse();
    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(STDIN_PATH)]
    } else {
        args.paths
    };

    let mut out = io::stdout().lock();
    let mut failed = false;

    for path in &paths {
        match cat_path(path, &mut out) {
            Ok(()) => {}
            Err(e) if is_broken_pipe(&e) => return ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("cat: {}: {}", path.display(), e);
                failed = true;
            }
        }
    }

    match out.flush() {
        Err(e) if !is_broken_pipe(&e) => {
            eprintln!("cat: write error: {}", e);
            ExitCode::FAILURE
        }
        _ if failed => ExitCode::FAILURE,
        _ => ExitCode::SUCCESS,
    }
}

fn cat_path(path: &Path, out: &mut impl Write) -> io::Result<()> {
    if path == Path::new(STDIN_PATH) {
        return cat(io::stdin().lock(), out);
    }
    cat(File::open(path)?, out)
}

fn cat(mut input: impl Read, out: &mut impl Write) -> io::Result<()> {
    io::copy(&mut input, out)?;
    Ok(())
}

fn is_broken_pipe(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::BrokenPipe
}
