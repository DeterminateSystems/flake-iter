use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Context;
use tracing::{debug, info};

use crate::{
    cmd::{
        get_output_json, nix_command, nix_command_pipe_no_output, nix_command_silence_output,
        SchemaOutput,
    },
    error::FlakeIterError,
};

// We need to include .drv paths when calculating the outputs so that Nix can build them
const INSPECT_FLAKE_REF: &str =
    "https://flakehub.com/f/DeterminateSystems/inspect/*#contents.includingOutputPaths";

/// Build all the derivations in the specified flake's outputs.
#[derive(Parser)]
pub(crate) struct Build {
    /// The specific Nix system to build for (otherwise infer the current system from arch/OS information).
    #[arg(short, long, env = "FLAKE_ITER_NIX_SYSTEM")]
    system: Option<String>,
}

impl Build {
    pub(crate) fn execute(&self, directory: PathBuf, verbose: bool) -> Result<(), FlakeIterError> {
        info!(
            dir = ?directory,
            "Building all derivations in the specified flake"
        );

        let current_system = self.system.clone().unwrap_or(get_nix_system());
        let flake_path = directory.clone().join("flake.nix");

        if !flake_path.exists() {
            return Err(FlakeIterError::Misc(format!(
                "No flake found at {}",
                directory.clone().display()
            )));
        }

        debug!(flake = ?flake_path, "Searching for derivations in flake outputs");

        let outputs: SchemaOutput = get_output_json(directory.clone(), INSPECT_FLAKE_REF)?;
        let derivations = outputs.derivations(&current_system);

        let num = derivations.len();

        if num > 0 {
            debug!(
                num = derivations.len(),
                system = current_system,
                "Discovered all flake derivation outputs"
            );

            info!("Building all unique derivations");

            let logged_in_to_flakehub = nix_command_pipe_no_output(&[
                "store",
                "info",
                "--store",
                "https://cache.flakehub.com",
            ])
            .is_ok();

            let mut n = 1;
            for (drv, (outputs, output_paths)) in derivations {
                let drv = format!("{}^{}", drv.display(), outputs.join(","));

                // Check to see if the outputs are already in the cache
                if logged_in_to_flakehub {
                    let args: Vec<String> = ["path-info", "--store", "https://cache.flakehub.com"]
                        .into_iter()
                        .map(String::from)
                        .chain(
                            output_paths
                                .iter()
                                .filter_map(|p| p.to_str())
                                .map(|v| v.to_string()),
                        )
                        .collect();

                    let arg_strs: Vec<&str> = args.iter().map(String::as_str).collect();
                    if nix_command_silence_output(&arg_strs).is_ok() {
                        info!("Skipping {drv}: its outputs are already in FlakeHub Cache.");
                        n += 1;
                        continue;
                    }
                }

                if verbose {
                    debug!(drv, "Building derivation {n} of {num}");
                    nix_command_pipe_no_output(&[
                        "build",
                        "--print-out-paths",
                        "--print-build-logs",
                        &drv,
                    ])
                    .wrap_err("failed to build derivation")?;
                } else {
                    info!("Building derivation {n} of {num}");
                    nix_command(&["build", "--print-out-paths", &drv])
                        .wrap_err("failed to build derivation")?;
                }
                n += 1;
            }

            info!("Successfully built all derivations");
        } else {
            info!("No derivations to build; exiting");
        }

        Ok(())
    }
}

fn get_nix_system() -> String {
    let arch = std::env::consts::ARCH;
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux",
        os => {
            panic!("os type {} not recognized", os);
        }
    };
    format!("{arch}-{os}")
}
