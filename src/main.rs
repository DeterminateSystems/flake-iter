use core::panic;
use std::collections::HashMap;

use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DevShell {
    name: Option<String>,
    r#type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: Option<String>,
    description: Option<String>,
    r#type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FlakeOutputs {
    #[serde(rename = "devShells")]
    dev_shells: HashMap<String, HashMap<String, DevShell>>,
    packages: HashMap<String, HashMap<String, Package>>,
}

#[derive(Debug, Parser)]
struct Cli {}

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

fn main() {
    let Cli { .. } = Cli::parse();

    let cmd_output = String::from_utf8(
        std::process::Command::new("nix")
            .args(["flake", "show", "--json"])
            .output()
            .expect("couldn't get command output")
            .stdout,
    )
    .expect("couldn't convert stdout to string");

    let outputs: FlakeOutputs =
        serde_json::from_str(&cmd_output).expect("couldn't deserialize string to json");

    let system = get_nix_system();

    for (sys, shell) in outputs.dev_shells {
        for (name, _) in shell {
            if sys == system {
                let output = format!(".#devShells.{system}.{name}");
                println!("Building {output}");
                std::process::Command::new("nix")
                    .args(["build", &output])
                    .output()
                    .expect("couldn't get command output")
                    .stdout;
                println!("Successfully built {output}");
            }
        }
    }

    for (sys, pkg) in outputs.packages {
        for (name, _) in pkg {
            if sys == system {
                let output = format!(".#packages.{system}.{name}");
                println!("Building {output}");
                std::process::Command::new("nix")
                    .args(["build", &output])
                    .output()
                    .expect("couldn't get command output")
                    .stdout;
                println!("Successfully built {output}");
            }
        }
    }
}
