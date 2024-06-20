use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct SchemaOutput {
    // ignore docs field
    pub inventory: HashMap<String, InventoryItem>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum InventoryItem {
    Parent(InventoryParent),
    Buildable(Buildable),
}

#[derive(Deserialize)]
pub struct InventoryParent {
    pub children: HashMap<String, InventoryItem>,
}

#[derive(Deserialize)]
pub struct Buildable {
    pub derivation: Option<PathBuf>,
    #[serde(rename = "forSystems")]
    pub for_systems: Option<Vec<String>>,
}
