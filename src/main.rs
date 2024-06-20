use std::{
    collections::HashSet,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Output, Stdio},
    time::Duration,
};

use clap::Parser;
use flake_iter::{
    flake::{Buildable, InventoryItem, Parent, SchemaOutput},
    get_nix_system, Cli, FlakeIterError,
};
use indicatif::ProgressBar;
use serde_json::Value;
use tracing::{debug, info, Level};
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), FlakeIterError> {
    let Cli { directory, verbose } = Cli::parse();
    let default_log_level = if verbose { Level::DEBUG } else { Level::INFO };

    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(default_log_level.into())
                .from_env_lossy(),
        )
        .init();

    info!(
        dir = ?directory,
        "Building all derivations in the specified flake"
    );

    let current_system = get_nix_system();
    let flake_path = directory.join("flake.nix");
    debug!(flake = ?flake_path, "Searching for derivations in flake outputs");

    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));

    bar.set_message("Assembling list of derivations to build");
    let outputs: SchemaOutput = get_output_json(directory)?;

    let mut derivations: HashSet<PathBuf> = HashSet::new();
    for item in outputs.inventory.values() {
        iterate_through_output_graph(&mut derivations, item, &current_system);
    }
    bar.finish_and_clear();

    debug!(
        num = derivations.len(),
        system = current_system,
        "Discovered all flake derivation outputs"
    );

    info!("Building all unique derivations");

    for drv in derivations {
        let drv = format!("{}^*", drv.display());
        debug!(drv, "Building derivation");
        let args = &["build", &drv];
        if verbose {
            nix_command_pipe(args)?;
        } else {
            nix_command(args)?;
        }
    }

    info!("Successfully built all derivations");

    Ok(())
}

fn iterate_through_output_graph(
    derivations: &mut HashSet<PathBuf>,
    item: &InventoryItem,
    current_system: &str,
) {
    match item {
        InventoryItem::Buildable(Buildable {
            derivation,
            for_systems,
        }) => {
            if let Some(for_systems) = for_systems {
                for system in for_systems {
                    if system == current_system {
                        if let Some(derivation) = derivation {
                            if derivations.insert(derivation.to_path_buf()) {
                                info!(drv = ?derivation, "Adding non-repeated derivation");
                            } else {
                                debug!(drv = ?derivation, "Skipping repeat derivation");
                            }
                        }
                    }
                }
            }
        }
        InventoryItem::Parent(Parent { children }) => {
            for item in children.values() {
                iterate_through_output_graph(derivations, item, current_system);
            }
        }
    }
}

fn get_output_json(dir: PathBuf) -> Result<SchemaOutput, FlakeIterError> {
    let metadata_json_output = nix_command(&[
        "flake",
        "metadata",
        "--json",
        "--no-write-lock-file",
        &dir.as_path().display().to_string(),
    ])?;
    let metadata_json: Value = serde_json::from_slice(&metadata_json_output.stdout)?;

    let flake_locked_url =
        metadata_json
            .get("url")
            .and_then(Value::as_str)
            .ok_or(FlakeIterError::Misc(String::from(
                "url field missing from flake metadata JSON",
            )))?;

    let drv =
        String::from("git+https://gist.github.com/bae261c8363414017fa4bdf8134ee53e.git#contents");

    let nix_eval_output = Command::new("nix")
        .args([
            "eval",
            "--json",
            "--no-write-lock-file",
            "--override-input",
            "flake",
            flake_locked_url,
            &drv,
        ])
        .output()?;

    let nix_eval_stdout = nix_eval_output.clone().stdout;

    if !nix_eval_output.status.success() {
        return Err(FlakeIterError::Misc(format!(
            "Failed to get flake outputs from tarball {}: {}",
            flake_locked_url,
            String::from_utf8(nix_eval_output.clone().stderr)?
        )));
    }

    let schema_output_json: SchemaOutput = serde_json::from_slice(&nix_eval_stdout)?;

    Ok(schema_output_json)
}

fn nix_command(args: &[&str]) -> Result<Output, FlakeIterError> {
    Ok(Command::new("nix").args(args).output()?)
}

fn nix_command_pipe(args: &[&str]) -> Result<(), FlakeIterError> {
    let mut cmd = Command::new("nix")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdout) = cmd.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(log) => println!("{}", log),
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }
    }

    Ok(())
}
