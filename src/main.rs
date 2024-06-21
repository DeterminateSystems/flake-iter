use clap::Parser;
use flake_iter::{
    cli::{Cli, FlakeIterCommand},
    FlakeIterError,
};

use tracing::Level;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), FlakeIterError> {
    let Cli { verbose, command } = Cli::parse();
    let default_log_level = if verbose { Level::DEBUG } else { Level::INFO };

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

    Ok(())
}
