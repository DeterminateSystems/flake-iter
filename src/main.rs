use core::panic;
use std::collections::HashMap;

use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Output {}

#[derive(Debug, Deserialize)]
struct FlakeOutputs {
    #[serde(rename = "devShells")]
    dev_shells: HashMap<String, HashMap<String, Output>>,
    packages: HashMap<String, HashMap<String, Output>>,
}

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short = 'd', long, default_value_t = true)]
    dev_shells: bool,

    #[arg(short = 'p', long, default_value_t = true)]
    packages: bool,
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

fn nix_build(output: &str) -> Result<(), FlakeIterError> {
    std::process::Command::new("nix")
        .args(["build", output])
        .output()?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum FlakeIterError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}

fn main() -> Result<(), FlakeIterError> {
    let Cli {
        dev_shells,
        packages,
    } = Cli::parse();

    if !dev_shells && !packages {
        println!("Nothing to build");
        return Ok(());
    }

    let flake_show_json = String::from_utf8(
        std::process::Command::new("nix")
            .args(["flake", "show", "--json"])
            .output()?
            .stdout,
    )?;

    let outputs: FlakeOutputs = serde_json::from_str(&flake_show_json)?;

    let system = get_nix_system();

    if packages {
        println!("Building package outputs");
        for (sys, pkg) in outputs.packages {
            for (name, _) in pkg {
                if sys == system {
                    let output = format!(".#packages.{system}.{name}");
                    println!("Building package output {name}");
                    nix_build(&output)?;
                    println!("Successfully built package {name}");
                }
            }
        }
        println!("Finished building package outputs");
    }

    if dev_shells {
        println!("Building dev shell outputs");
        for (sys, shell) in outputs.dev_shells {
            for (name, _) in shell {
                if sys == system {
                    let output = format!(".#devShells.{system}.{name}");
                    println!("Building dev shell {name}");
                    nix_build(&output)?;
                    println!("Successfully built dev shell {name}");
                }
            }
        }
        println!("Finished building dev shell outputs");
    }

    Ok(())
}
