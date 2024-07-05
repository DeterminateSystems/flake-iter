use std::process::ExitCode;

use flake_iter::cli::Cli;

fn main() -> color_eyre::Result<ExitCode> {
    Cli::execute()
}
