use std::path::PathBuf;

use anyhow::Result;
use binlang::generate_c;
use clap::Parser as _;

#[derive(clap::Parser)]
struct Cli {
    #[clap(index = 1, help = "Input binlang file")]
    input: PathBuf,
    #[clap(short, long, default_value = "gen/src", help = "Output directory")]
    output: PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Cli::parse();

    let filename = args.input.file_stem().unwrap().to_string_lossy();
    let source = std::fs::read_to_string(&args.input)?;
    generate_c(&filename, &source, &args.output)?;

    println!("Successfully generated to {:?}", args.output);

    Ok(())
}
