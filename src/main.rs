use clap::Parser;
use eyre::Result;
use std::{fs, path::PathBuf};
use ultravis::parser::parse;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The file to output the video to.
    #[arg(short, long, default_value = "video.mp4")]
    pub output: PathBuf,

    /// The .uvis file to create a visualizer on.
    pub input: PathBuf,
}

// Why is this a seperate function? Lifetimes.
fn run<'a>(input: &'a str, file_path: &PathBuf, output: PathBuf) -> Result<()> {
    let (data, opts) = parse(file_path, input)?;

    ultravis::mux::mux_video(&data, &opts, output)?;

    Ok(())
}

pub fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let data = fs::read_to_string(&cli.input)?;

    run(&data, &cli.input, cli.output)?;

    Ok(())
}
