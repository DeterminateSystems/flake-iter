use core::panic;
use std::{collections::HashMap, io::Write, path::PathBuf, process::Command};

use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const FLAKE_URL_PLACEHOLDER_UUID: &str = "c9026fc0-ced9-48e0-aa3c-fc86c4c86df1";

/// A tool for building all the derivations in a flake's output.
#[derive(Parser)]
struct Cli {
    #[arg(short = 'd', long, default_value = ".")]
    directory: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct SchemaOutput {
    // ignore docs
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
    derivation: PathBuf,
    #[serde(rename = "forSystems")]
    for_systems: Vec<String>,
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

    for (name, item) in outputs.inventory {
        handle_item(&name, &item, &current_system);
    }

    Ok(())
}

fn handle_item(name: &str, item: &InventoryItem, current_system: &str) {
    match item {
        InventoryItem::Buildable(buildable) => {
            for system in &buildable.for_systems {
                if system == current_system {
                    println!("name: {name}, buildable: {:?}", buildable.derivation);
                }
            }
        }
        InventoryItem::Parent(parent) => {
            for (name, item) in &parent.children {
                handle_item(&name, &item, &current_system);
            }
        }
    }
}

fn get_output_json(dir: PathBuf) -> Result<SchemaOutput, FlakeIterError> {
    let metadata_json_output = Command::new("nix")
        .args([
            "flake",
            "metadata",
            "--json",
            "--no-write-lock-file",
            &dir.as_path().display().to_string(),
        ])
        .output()?;
    let metadata_json: Value = serde_json::from_slice(&metadata_json_output.stdout)?;

    let flake_locked_url =
        metadata_json
            .get("url")
            .and_then(Value::as_str)
            .ok_or(FlakeIterError::Misc(String::from(
                "url field missing from flake metadata JSON",
            )))?;

    println!("flake locked URL: {flake_locked_url}");

    let tempdir = tempfile::Builder::new()
        .prefix("flakehub_push_outputs")
        .tempdir()?;

    println!("temp directory: {tempdir:?}");

    let flake_contents =
        include_str!("./mixed-flake.nix").replace(FLAKE_URL_PLACEHOLDER_UUID, &flake_locked_url);

    let flake_path = tempdir.path().join("flake.nix");
    println!("flake output path: {flake_path:?}");

    let mut flake = std::fs::File::create(flake_path)?;
    flake.write_all(flake_contents.as_bytes())?;

    println!("temporary flake.nix created");

    let drv = format!("{}#contents", tempdir.path().display());
    println!("derivation: {drv}");

    let nix_eval_output = Command::new("nix")
        .args(["eval", "--json", "--no-write-lock-file", &drv])
        .output()?;

    let nix_eval_stdout = nix_eval_output.stdout;

    println!(
        "nix eval output: {}",
        String::from_utf8(nix_eval_stdout.clone())?
    );

    if !nix_eval_output.status.success() {
        return Err(FlakeIterError::Misc(format!(
            "Failed to get flake outputs from tarball {}: {}",
            flake_locked_url,
            String::from_utf8(nix_eval_output.stderr)?
        )));
    }

    let schema_output_json: SchemaOutput = serde_json::from_slice(&nix_eval_stdout)?;

    std::fs::remove_dir_all(tempdir)?;

    Ok(schema_output_json)
}
