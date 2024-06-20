use std::path::PathBuf;

use clap::Parser;

/// A tool for building all the derivations in a flake's output.
#[derive(Parser)]
pub struct Cli {
    /// The directory of the target flake.
    #[arg(short = 'd', long, default_value = ".")]
    pub directory: PathBuf,
}
