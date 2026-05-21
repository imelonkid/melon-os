use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::manifest::Manifest;
use crate::validation::validate_pack;

/// Represents a loaded scenario pack.
#[derive(Debug, Clone)]
pub struct ScenarioPack {
    pub path: PathBuf,
    pub manifest: Manifest,
}

/// Discover all scenario packs in a directory.
pub fn discover_packs(base_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut packs = Vec::new();
    if !base_dir.is_dir() {
        return Ok(packs);
    }

    for entry in std::fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("manifest.yaml").exists() {
            packs.push(path);
        }
    }

    Ok(packs)
}

/// Load a scenario pack from a directory.
pub fn load_pack(dir: &Path) -> Result<ScenarioPack> {
    let errors = validate_pack(dir)?;
    if !errors.is_empty() {
        anyhow::bail!("Pack validation errors: {:?}", errors);
    }

    let manifest_path = dir.join("manifest.yaml");
    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = serde_yaml::from_str(&content)?;

    Ok(ScenarioPack {
        path: dir.to_path_buf(),
        manifest,
    })
}

/// Read a file within a scenario pack.
pub fn read_pack_file(dir: &Path, relative: &str) -> Result<String> {
    let path = dir.join(relative);
    std::fs::read_to_string(&path).map_err(|e| anyhow::anyhow!("Failed to read {:?}: {}", path, e))
}
