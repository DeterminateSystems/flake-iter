use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde_json::Value;

/// Write the systems/runners array to the file at `$GITHUB_OUTPUT`.
#[derive(Parser)]
pub struct Systems {
    /// The directory of the target flake.
    #[arg(short, long, env = "FLAKE_ITER_DIRECTORY", default_value = ".")]
    pub directory: PathBuf,

    /// A mapping of GitHub Actions runners to Nix systems.
    /// Example: {"aarch64-darwin": "macos-latest-xlarge"}
    #[arg(short, long, env = "FLAKE_ITER_RUNNER_MAP")]
    pub runner_map: Option<Value>,
}

/// Build all
#[derive(Parser)]
pub struct Build {
    /// The directory of the target flake.
    #[arg(short, long, env = "FLAKE_ITER_DIRECTORY", default_value = ".")]
    pub directory: PathBuf,
}

#[derive(Subcommand)]
pub enum FlakeIterCommand {
    Build(Build),
    Systems(Systems),
}

/// A tool for working with flake outputs.
#[derive(Parser)]
pub struct Cli {
    /// Whether to display all Nix build output.
    #[arg(short, long, env = "FLAKE_ITER_VERBOSE", default_value_t = false)]
    pub verbose: bool,

    #[clap(subcommand)]
    pub command: FlakeIterCommand,
}
