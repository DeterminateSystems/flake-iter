use std::path::PathBuf;

use clap::Parser;

/// A tool for building all the derivations in a flake's output.
#[derive(Parser)]
pub struct Cli {
    /// The directory of the target flake.
    #[arg(short, long, env = "FLAKE_ITER_DIRECTORY", default_value = ".")]
    pub directory: PathBuf,

    /// Whether to display all Nix build output.
    #[arg(short, long, env = "FLAKE_ITER_VERBOSE", default_value_t = false)]
    pub verbose: bool,

    /// Whether to write the systems matrix to the file at `$GITHUB_OUTPUT`.
    #[arg(short, long, env = "FLAKE_ITER_MATRIX", default_value_t = false)]
    pub matrix: bool,
}
