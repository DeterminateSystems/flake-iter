use core::panic;
use std::{collections::HashMap, process::exit};

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
    #[arg(short = 'd', long, default_value_t = false)]
    dev_shells: bool,

    #[arg(short = 'p', long, default_value_t = false)]
    packages: bool,

    #[arg(short = 'b', long, default_value_t = false)]
    build: bool,
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

fn nix_build(output: &str) {
    std::process::Command::new("nix")
        .args(["build", output])
        .output()
        .expect("couldn't get command output");
}

fn main() {
    let Cli {
        dev_shells,
        packages,
        build,
    } = Cli::parse();

    if !build || (!dev_shells && !packages) {
        println!("Nothing to build");
        exit(1);
    }

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

    if dev_shells && build {
        println!("Building dev shell outputs\n");
        for (sys, shell) in outputs.dev_shells {
            for (name, _) in shell {
                if sys == system {
                    let output = format!(".#devShells.{system}.{name}");
                    println!("Building dev shell {name}");
                    nix_build(&output);
                    println!("Successfully built dev shell {name}");
                }
            }
        }
    }

    if packages && build {
        println!("Building package outputs\n");
        for (sys, pkg) in outputs.packages {
            for (name, _) in pkg {
                if sys == system {
                    let output = format!(".#packages.{system}.{name}");
                    println!("Building package {name}");
                    nix_build(&output);
                    println!("Successfully built package {name}");
                }
            }
        }
    }
}
