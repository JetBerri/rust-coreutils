use std::fs;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    path: std::path::PathBuf,
}
fn main(){
    let args = Cli::parse();
    let content = fs::read_to_string(args.path)
                .expect("There has been an error while reading your file");
    println!("{}", content);
}