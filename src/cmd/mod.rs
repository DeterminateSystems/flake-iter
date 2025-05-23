mod build;
mod systems;

pub(crate) use build::Build;
use color_eyre::eyre::Context;
pub(crate) use systems::Systems;

use tracing::debug;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::FlakeIterError;

#[derive(Deserialize)]
struct SchemaOutput {
    // ignore docs field
    inventory: HashMap<String, InventoryItem>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum InventoryItem {
    Parent(Parent),
    Buildable(Buildable),
}

#[derive(Deserialize)]
pub(super) struct Parent {
    children: HashMap<String, InventoryItem>,
}

#[derive(Deserialize)]
pub(super) struct Buildable {
    derivation: Option<PathBuf>,
    #[serde(rename = "forSystems")]
    for_systems: Option<Vec<String>>,
    #[serde(default)]
    outputs: HashMap<String, PathBuf>,
}

#[derive(Eq, Hash, PartialEq, Serialize)]
struct SystemAndRunner {
    #[serde(rename = "nix-system")]
    nix_system: String,
    runner: String,
}

impl SchemaOutput {
    fn derivations(&self, current_system: &str) -> HashMap<PathBuf, Vec<String>> {
        let mut derivations: HashMap<PathBuf, Vec<String>> = HashMap::new();
        for item in self.inventory.values() {
            accumulate_derivations(&mut derivations, item, current_system);
        }
        derivations
    }

    fn systems(&self, runner_map: HashMap<String, String>) -> (Vec<SystemAndRunner>, Vec<String>) {
        let mut systems: HashSet<SystemAndRunner> = HashSet::new();
        let mut systems_without_runners: HashSet<String> = HashSet::new();

        for item in self.inventory.values() {
            accumulate_systems(
                &mut systems,
                &mut systems_without_runners,
                item,
                &runner_map,
            );
        }

        (
            Vec::from_iter(systems),
            Vec::from_iter(systems_without_runners),
        )
    }
}

fn accumulate_derivations(
    derivations: &mut HashMap<PathBuf, Vec<String>>,
    item: &InventoryItem,
    current_system: &str,
) {
    match item {
        InventoryItem::Buildable(Buildable {
            derivation,
            for_systems,
            outputs,
        }) => {
            if let Some(for_systems) = for_systems {
                for system in for_systems {
                    if system == current_system {
                        if let Some(derivation) = derivation {
                            if derivations
                                .insert(derivation.to_path_buf(), outputs.keys().cloned().collect())
                                .is_some()
                            {
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
                accumulate_derivations(derivations, item, current_system);
            }
        }
    }
}

fn accumulate_systems(
    systems: &mut HashSet<SystemAndRunner>,
    systems_without_runners: &mut HashSet<String>,
    item: &InventoryItem,
    runner_map: &HashMap<String, String>,
) {
    match item {
        InventoryItem::Buildable(Buildable { for_systems, .. }) => {
            if let Some(for_systems) = for_systems {
                for system in for_systems {
                    if let Some(runner) = runner_map.get(system) {
                        systems.insert(SystemAndRunner {
                            runner: String::from(runner),
                            nix_system: String::from(system),
                        });
                    } else {
                        systems_without_runners.insert(String::from(system));
                    }
                }
            }
        }
        InventoryItem::Parent(Parent { children }) => {
            for item in children.values() {
                accumulate_systems(systems, systems_without_runners, item, runner_map);
            }
        }
    }
}

fn get_output_json(dir: PathBuf, inspect_flake_ref: &str) -> Result<SchemaOutput, FlakeIterError> {
    // This acts as a quick pre-check. If this fails, then assembling the list of derivations
    // is bound to fail.
    nix_command(&["flake", "show"]).wrap_err("failed to show flake outputs")?;

    let flake_path = dir.as_path().display().to_string();

    debug!(flake = flake_path, "Fetching flake metadata");

    let metadata_json_output = nix_command(&[
        "flake",
        "metadata",
        "--json",
        "--no-write-lock-file",
        &flake_path,
    ])
    .wrap_err("failed to get flake metadata")?;

    debug!(flake = flake_path, "Fetched flake metadata");

    let metadata_json: Value = serde_json::from_slice(&metadata_json_output.stdout)?;

    let flake_locked_url =
        metadata_json
            .get("url")
            .and_then(Value::as_str)
            .ok_or(FlakeIterError::Misc(String::from(
                "url field missing from flake metadata JSON",
            )))?;

    debug!(url = flake_locked_url, "Flake locked URL");

    let nix_eval_output = nix_command_pipe_with_output(&[
        "eval",
        "--json",
        "--no-write-lock-file",
        "--override-input",
        "flake",
        flake_locked_url,
        inspect_flake_ref,
    ])?;

    let nix_eval_stdout = nix_eval_output.clone().stdout;

    if !nix_eval_output.status.success() {
        return Err(FlakeIterError::Misc(format!(
            "Failed to get flake outputs from tarball {}: {}",
            flake_locked_url,
            String::from_utf8(nix_eval_output.clone().stderr)?
        )));
    }

    Ok(serde_json::from_slice(&nix_eval_stdout)?)
}

fn nix_command(args: &[&str]) -> Result<Output, FlakeIterError> {
    let output = Command::new("nix")
        .args(args)
        .output()
        .wrap_err("nix command invocation failed")?;

    if output.status.success() {
        Ok(output)
    } else {
        Err(FlakeIterError::Misc(output_to_string(output)))
    }
}

fn nix_command_pipe_with_output(args: &[&str]) -> Result<Output, FlakeIterError> {
    let output = Command::new("nix")
        .args(args)
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .wrap_err("failed to spawn Nix command")?
        .wait_with_output()
        .wrap_err("failed to wait for Nix command output")?;

    if output.status.success() {
        Ok(output)
    } else {
        Err(FlakeIterError::Misc(output_to_string(output)))
    }
}

fn nix_command_pipe_no_output(args: &[&str]) -> Result<(), FlakeIterError> {
    let output = Command::new("nix")
        .args(args)
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
        .wrap_err("failed to spawn Nix command")?
        .wait_with_output()
        .wrap_err("failed to wait for Nix command output")?;

    if output.status.success() {
        Ok(())
    } else {
        Err(FlakeIterError::Misc(output_to_string(output)))
    }
}

fn output_to_string(output: Output) -> String {
    let mut s = String::new();

    if !output.stdout.is_empty() {
        s.push_str(&String::from_utf8_lossy(&output.stdout));
    }

    if !output.stderr.is_empty() {
        s.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    s
}
