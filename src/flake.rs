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
    pub fn systems(&self) -> Vec<SystemAndRunner> {
        let mut systems: HashSet<SystemAndRunner> = HashSet::new();

        for item in self.inventory.values() {
            iterate(&mut systems, item);
        }

        Vec::from_iter(systems)
    }
}

fn iterate(systems: &mut HashSet<SystemAndRunner>, item: &InventoryItem) {
    match item {
        InventoryItem::Buildable(Buildable {
            derivation,
            for_systems,
        }) => {
            if let Some(for_systems) = for_systems {
                for system in for_systems {
                    if derivation.is_some() {
                        // TODO: make this more elegant
                        if system == X86_64_LINUX {
                            systems.insert(SystemAndRunner {
                                runner: String::from(X86_64_LINUX_RUNNER),
                                nix_system: String::from(X86_64_LINUX),
                            });
                        }

                        if system == AARCH64_LINUX {
                            systems.insert(SystemAndRunner {
                                runner: String::from(AARCH64_LINUX_RUNNER),
                                nix_system: String::from(AARCH64_LINUX),
                            });
                        }

                        if system == X86_64_DARWIN {
                            systems.insert(SystemAndRunner {
                                runner: String::from(X86_64_DARWIN_RUNNER),
                                nix_system: String::from(X86_64_DARWIN),
                            });
                        }

                        if system == AARCH64_DARWIN {
                            systems.insert(SystemAndRunner {
                                runner: String::from(AARCH64_DARWIN_RUNNER),
                                nix_system: String::from(AARCH64_DARWIN),
                            });
                        }
                    }
                }
            }
        }
        InventoryItem::Parent(Parent { children }) => {
            for item in children.values() {
                iterate(systems, item);
            }
        }
    }
}
