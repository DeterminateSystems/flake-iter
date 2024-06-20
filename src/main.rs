use core::panic;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    process::{Command, Output},
};

use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A tool for building all the derivations in a flake's output.
#[derive(Parser)]
struct Cli {
    #[arg(short = 'd', long, default_value = ".")]
    directory: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct SchemaOutput {
    // ignore docs field
    inventory: HashMap<String, InventoryItem>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum InventoryItem {
    Parent(InventoryParent),
    Buildable(Buildable),
}

#[derive(Serialize, Deserialize)]
struct InventoryParent {
    children: HashMap<String, InventoryItem>,
}

#[derive(Serialize, Deserialize)]
struct Buildable {
    derivation: Option<PathBuf>,
    #[serde(rename = "forSystems")]
    for_systems: Option<Vec<String>>,
}

#[allow(dead_code)]
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

#[derive(Debug, thiserror::Error)]
enum FlakeIterError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Misc(String),

    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}

fn main() -> Result<(), FlakeIterError> {
    let Cli { directory } = Cli::parse();

    let current_system = get_nix_system();

    let outputs: SchemaOutput = get_output_json(directory)?;

    let mut derivations: HashSet<PathBuf> = HashSet::new();

    for item in outputs.inventory.values() {
        handle_item(&mut derivations, item, &current_system);
    }

    println!("Derivations to build:");
    for drv in derivations {
        println!("{drv:?}");
    }

    Ok(())
}

fn handle_item(derivations: &mut HashSet<PathBuf>, item: &InventoryItem, current_system: &str) {
    match item {
        InventoryItem::Buildable(Buildable {
            derivation,
            for_systems,
        }) => {
            if let Some(for_systems) = for_systems {
                for system in for_systems {
                    if system == current_system {
                        if let Some(derivation) = derivation {
                            derivations.insert(derivation.to_path_buf());
                        }
                    }
                }
            }
        }
        InventoryItem::Parent(parent) => {
            for item in parent.children.values() {
                handle_item(derivations, item, current_system);
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
