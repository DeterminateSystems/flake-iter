use std::{io::IsTerminal, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use tracing::Level;
use tracing_subscriber::EnvFilter;

use crate::cmd::{Build, Systems};

#[derive(Subcommand)]
enum FlakeIterCommand {
    Build(Build),
    Systems(Systems),
}

/// A tool for working with flake outputs.
#[derive(Parser)]
pub struct Cli {
    /// Whether to display all Nix build output.
    #[arg(
        short,
        long,
        env = "FLAKE_ITER_VERBOSE",
        default_value_t = false,
        global = true
    )]
    verbose: bool,

    /// The directory of the target flake.
    #[arg(
        short,
        long,
        env = "FLAKE_ITER_DIRECTORY",
        default_value = ".",
        global = true
    )]
    directory: PathBuf,

    #[clap(subcommand)]
    command: FlakeIterCommand,
}

impl Cli {
    pub fn execute() -> color_eyre::Result<ExitCode> {
        let Cli {
            verbose,
            directory,
            command,
        } = Cli::parse();

        let default_log_level = if verbose { Level::DEBUG } else { Level::INFO };

        color_eyre::config::HookBuilder::default()
            // flake-iter is a private repo so we direct people to the ci repo for reporting issues
            .issue_url("https://github.com/DeterminateSystems/ci/issues/new")
            .add_issue_metadata("version", env!("CARGO_PKG_VERSION"))
            .add_issue_metadata("os", std::env::consts::OS)
            .add_issue_metadata("arch", std::env::consts::ARCH)
            .theme(if !std::io::stderr().is_terminal() {
                color_eyre::config::Theme::new()
            } else {
                color_eyre::config::Theme::dark()
            })
            .install()?;

        tracing_subscriber::fmt()
            .with_ansi(true)
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(default_log_level.into())
                    .from_env_lossy(),
            )
            .init();

        match command {
            FlakeIterCommand::Build(build) => build.execute(directory, verbose)?,
            FlakeIterCommand::Systems(systems) => systems.execute(directory)?,
        }

        Ok(ExitCode::SUCCESS)
    }
}
