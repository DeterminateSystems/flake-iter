mod build;
mod systems;

pub use build::Build;
pub use systems::Systems;

use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::FlakeIterError;

const X86_64_LINUX: &str = "x86_64-linux";
const X86_64_LINUX_RUNNER: &str = "UbuntuLatest32Cores128G";
const AARCH64_LINUX: &str = "aarch64-linux";
const AARCH64_LINUX_RUNNER: &str = "UbuntuLatest32Cores128GArm";
const X86_64_DARWIN: &str = "x86_64-darwin";
const X86_64_DARWIN_RUNNER: &str = "macos-latest-xlarge";
const AARCH64_DARWIN: &str = "aarch64-darwin";
const AARCH64_DARWIN_RUNNER: &str = "macos-latest-xlarge";

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
    runner: String,
    #[serde(rename = "nix-system")]
    nix_system: String,
}

impl SchemaOutput {
    fn systems(&self, runner_map: &Option<HashMap<String, String>>) -> Vec<SystemAndRunner> {
        let runner_map = runner_map.clone().unwrap_or(HashMap::from([
            (
                String::from(X86_64_LINUX),
                String::from(X86_64_LINUX_RUNNER),
            ),
            (
                String::from(AARCH64_LINUX),
                String::from(AARCH64_LINUX_RUNNER),
            ),
            (
                String::from(X86_64_DARWIN),
                String::from(X86_64_DARWIN_RUNNER),
            ),
            (
                String::from(AARCH64_DARWIN),
                String::from(AARCH64_DARWIN_RUNNER),
            ),
        ]));

        let mut systems: HashSet<SystemAndRunner> = HashSet::new();

        for item in self.inventory.values() {
            iterate(&mut systems, item, &runner_map);
        }

        Vec::from_iter(systems)
    }
}

fn iterate(
    systems: &mut HashSet<SystemAndRunner>,
    item: &InventoryItem,
    runner_map: &HashMap<String, String>,
) {
    match item {
        InventoryItem::Buildable(Buildable {
            derivation,
            for_systems,
        }) => {
            if let Some(for_systems) = for_systems {
                for system in for_systems {
                    if derivation.is_some() {
                        if let Some(runner) = runner_map.get(system) {
                            systems.insert(SystemAndRunner {
                                runner: String::from(runner),
                                nix_system: String::from(system),
                            });
                        }
                    }
                }
            }
        }
        InventoryItem::Parent(Parent { children }) => {
            for item in children.values() {
                iterate(systems, item, runner_map);
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
