use core::panic;
use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct FlakeOutputs {
    #[serde(rename = "devShells")]
    dev_shells: Option<HashMap<String, HashMap<String, Value>>>,
    #[serde(rename = "dockerImages")]
    docker_images: Option<HashMap<String, HashMap<String, Value>>>,
    packages: Option<HashMap<String, HashMap<String, Value>>>,
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
    let flake_show_json = String::from_utf8(
        std::process::Command::new("nix")
            .args(["flake", "show", "--json"])
            .output()?
            .stdout,
    )?;
    let outputs: FlakeOutputs = serde_json::from_str(&flake_show_json)?;
    let system = get_nix_system();

    // Package outputs
    if let Some(packages) = outputs.packages {
        println!("Building package outputs");
        for (sys, pkg) in packages {
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

    // Dev shell outputs
    if let Some(dev_shells) = outputs.dev_shells {
        println!("Building dev shell outputs");
        for (sys, shell) in dev_shells {
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

    // Docker image outputs
    if let Some(docker_images) = outputs.docker_images {
        println!("Building Docker image outputs");
        for (sys, docker_image) in docker_images {
            for (name, _) in docker_image {
                if sys == system {
                    let output = format!(".#devShells.{system}.{name}");
                    println!("Building Docker image {name}");
                    nix_build(&output)?;
                    println!("Successfully built Docker image {name}");
                }
            }
        }
        println!("Finished building Docker image outputs");
    }

    Ok(())
}
