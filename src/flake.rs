use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

// TODO: make this more customizable
const X86_64_LINUX: &str = "x86_64-linux";
const X86_64_LINUX_RUNNER: &str = "UbuntuLatest32Cores128G";
const AARCH64_LINUX: &str = "aarch64-linux";
const AARCH64_LINUX_RUNNER: &str = "UbuntuLatest32Cores128GArm";
const X86_64_DARWIN: &str = "x86_64-darwin";
const X86_64_DARWIN_RUNNER: &str = "macos-latest-xlarge";
const AARCH64_DARWIN: &str = "aarch64-darwin";
const AARCH64_DARWIN_RUNNER: &str = "macos-latest-xlarge";

#[derive(Deserialize)]
pub struct SchemaOutput {
    // ignore docs field
    pub inventory: HashMap<String, InventoryItem>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum InventoryItem {
    Parent(Parent),
    Buildable(Buildable),
}

#[derive(Deserialize)]
pub struct Parent {
    pub children: HashMap<String, InventoryItem>,
}

#[derive(Deserialize)]
pub struct Buildable {
    pub derivation: Option<PathBuf>,
    #[serde(rename = "forSystems")]
    pub for_systems: Option<Vec<String>>,
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub struct SystemAndRunner {
    runner: String,
    #[serde(rename = "nix-system")]
    nix_system: String,
}

impl SchemaOutput {
    pub fn systems(&self, runner_map: &Option<HashMap<String, String>>) -> Vec<SystemAndRunner> {
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
