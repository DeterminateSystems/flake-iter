use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

use clap::Parser;
use tracing::{debug, info, warn};

use crate::{
    cmd::{get_output_json, SchemaOutput},
    error::FlakeIterError,
};

const GITHUB_OUTPUT_KEY: &str = "systems";

// We don't need the .drv paths in the JSON as we don't need to build anything
const INSPECT_FLAKE_REF: &str =
    "https://flakehub.com/f/DeterminateSystems/inspect/*#contents.excludingOutputPaths";

/// Write the systems/runners array to the file at `$GITHUB_OUTPUT`.
#[derive(Parser)]
pub(crate) struct Systems {
    /// A mapping of GitHub Actions runners to Nix systems.
    /// Example: {"aarch64-darwin": "macos-latest-xlarge"}
    #[arg(short, long, env = "FLAKE_ITER_RUNNER_MAP")]
    runner_map: String,
}

impl Systems {
    pub(crate) fn execute(&self, directory: PathBuf) -> Result<(), FlakeIterError> {
        let runner_map: HashMap<String, String> = serde_json::from_str(&self.runner_map)?;

        info!("Generating systems matrix for GitHub Actions");
        let outputs: SchemaOutput = get_output_json(directory.clone(), INSPECT_FLAKE_REF)?;

        let (systems, systems_without_runners) = outputs.systems(runner_map);

        if systems.is_empty() {
            warn!("Flake does not include a nix system.");
        }

        for system in systems_without_runners {
            warn!("Flake contains derivation outputs for Nix system `{system}` but no runner is specified for that system");
        }

        let matrix_str = serde_json::to_string(&systems)?;
        let output_str = format!("{GITHUB_OUTPUT_KEY}={}", matrix_str);
        debug!("Output string: {output_str}");

        let github_output_file = std::env::var("GITHUB_OUTPUT")?;
        debug!(
            "Writing output string to GITHUB_OUTPUT file at {}",
            &github_output_file
        );
        let mut file = File::create(PathBuf::from(&github_output_file))?;
        file.write_all(output_str.as_bytes())?;
        debug!("Output string written to {}", &github_output_file);

        info!("Successfully wrote systems matrix");

        Ok(())
    }
}
