use clap::{Parser, Subcommand};

use crate::cmd::{Build, Systems};

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
