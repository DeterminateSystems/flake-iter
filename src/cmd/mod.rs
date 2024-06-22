mod build;
mod systems;

pub use build::Build;
pub use systems::Systems;
use tracing::warn;

use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::FlakeIterError;

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
}

#[derive(Eq, Hash, PartialEq, Serialize)]
struct SystemAndRunner {
    #[serde(rename = "nix-system")]
    nix_system: String,
    runner: String,
}

impl SchemaOutput {
    fn systems(&self, runner_map: HashMap<String, String>) -> Vec<SystemAndRunner> {
        let mut systems: HashSet<SystemAndRunner> = HashSet::new();
        let mut systems_without_runners: HashSet<String> = HashSet::new();

        for item in self.inventory.values() {
            iterate(
                &mut systems,
                &mut systems_without_runners,
                item,
                &runner_map,
            );
        }

        for system in systems_without_runners {
            warn!("Flake contains derivation outputs for Nix system `{system}` but no runner is specified for that system")
        }

        Vec::from_iter(systems)
    }
}

fn iterate(
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
                iterate(systems, systems_without_runners, item, runner_map);
            }
        }
    }
}

fn get_output_json(dir: PathBuf, inspect_flake_ref: &str) -> Result<SchemaOutput, FlakeIterError> {
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

    let nix_eval_output = Command::new("nix")
        .args([
            "eval",
            "--json",
            "--no-write-lock-file",
            "--override-input",
            "flake",
            flake_locked_url,
            inspect_flake_ref,
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
