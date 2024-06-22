use std::{io::IsTerminal, process::ExitCode};

use clap::Parser;
use flake_iter::cli::{Cli, FlakeIterCommand};
use tracing::Level;
use tracing_subscriber::EnvFilter;

fn main() -> color_eyre::Result<ExitCode> {
    let Cli { verbose, command } = Cli::parse();
    let default_log_level = if verbose { Level::DEBUG } else { Level::INFO };

    color_eyre::config::HookBuilder::default()
        .issue_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues/new"))
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
        FlakeIterCommand::Build(build) => build.execute(verbose)?,
        FlakeIterCommand::Systems(systems) => systems.execute()?,
    }

    Ok(ExitCode::SUCCESS)
}
