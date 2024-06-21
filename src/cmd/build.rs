use std::{collections::HashSet, path::PathBuf, time::Duration};

use clap::Parser;
use indicatif::ProgressBar;
use tracing::{debug, info};

use crate::{
    cmd::{get_output_json, nix_command, nix_command_pipe, SchemaOutput},
    FlakeIterError,
};

use super::{Buildable, InventoryItem, Parent};

// We need to include .drv paths when calculating the outputs so that Nix can build them
const INSPECT_FLAKE_REF: &str =
    "https://flakehub.com/f/DeterminateSystems/inspect/*#contents.includingOutputPaths";

/// Build all the derivations in the specified flake's outputs.
#[derive(Parser)]
pub struct Build {
    /// The directory of the target flake.
    #[arg(short, long, env = "FLAKE_ITER_DIRECTORY", default_value = ".")]
    directory: PathBuf,
}

impl Build {
    pub fn execute(&self, verbose: bool) -> Result<(), FlakeIterError> {
        info!(
            dir = ?self.directory,
            "Building all derivations in the specified flake"
        );

        let current_system = get_nix_system();
        let flake_path = self.directory.clone().join("flake.nix");

        if !flake_path.exists() {
            return Err(FlakeIterError::Misc(format!(
                "No flake found at {}",
                self.directory.clone().display()
            )));
        }

        debug!(flake = ?flake_path, "Searching for derivations in flake outputs");

        let bar = ProgressBar::new_spinner();
        bar.enable_steady_tick(Duration::from_millis(100));

        bar.set_message("Assembling list of derivations to build");
        let outputs: SchemaOutput = get_output_json(self.directory.clone(), INSPECT_FLAKE_REF)?;

        let mut derivations: HashSet<PathBuf> = HashSet::new();
        for item in outputs.inventory.values() {
            iterate_through_output_graph(&mut derivations, item, &current_system);
        }
        bar.finish_and_clear();

        let num = derivations.len();

        if num > 0 {
            debug!(
                num = derivations.len(),
                system = current_system,
                "Discovered all flake derivation outputs"
            );

            info!("Building all unique derivations");

            let mut n = 1;
            for drv in derivations {
                let drv = format!("{}^*", drv.display());
                if verbose {
                    debug!(drv, "Building derivation {n} of {num}");
                } else {
                    info!("Building derivation {n} of {num}");
                }
                let args = &["build", &drv];
                if verbose {
                    nix_command_pipe(args)?;
                } else {
                    nix_command(args)?;
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
                                debug!(drv = ?derivation, "Adding non-repeated derivation");
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
