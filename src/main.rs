use std::collections::HashMap;

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

fn main() {
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

    println!("{:?}", outputs);
}
