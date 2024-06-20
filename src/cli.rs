use std::path::PathBuf;

use clap::Parser;

/// A tool for building all the derivations in a flake's output.
#[derive(Parser)]
pub struct Cli {
    /// The directory of the target flake.
    #[arg(short, long, default_value = ".")]
    pub directory: PathBuf,

    /// Whether to display all Nix build output.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
