use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::AppState;

#[derive(Serialize)]
struct PackSummary {
    id: String,
    name: String,
    version: String,
    description: Option<String>,
    path: String,
    valid: bool,
    validation_errors: Vec<String>,
}

pub(crate) struct ManifestSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/packs", get(list_packs))
}

async fn list_packs(State(state): State<AppState>) -> Json<Vec<PackSummary>> {
    let packs = match melon_scenario::pack::discover_packs(&state.scenarios_dir) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to discover packs: {}", e);
            return Json(Vec::new());
        }
    };

    let mut summaries = Vec::new();
    for pack_path in &packs {
        let summary = read_manifest_summary(pack_path);
        let validation_errors = melon_scenario::validation::validate_pack(pack_path)
            .unwrap_or_else(|e| vec![format!("failed to validate pack: {}", e)]);

        summaries.push(PackSummary {
            id: summary.id,
            name: summary.name,
            version: summary.version,
            description: summary.description,
            path: pack_path.to_string_lossy().to_string(),
            valid: validation_errors.is_empty(),
            validation_errors,
        });
    }

    Json(summaries)
}

pub(crate) fn resolve_pack_dir_by_id(
    scenarios_dir: &Path,
    pack_id: &str,
) -> std::io::Result<Option<PathBuf>> {
    let entries = std::fs::read_dir(scenarios_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() || !path.join("manifest.yaml").exists() {
            continue;
        }

        let summary = read_manifest_summary(&path);
        if summary.id == pack_id {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn read_manifest_summary(pack_path: &Path) -> ManifestSummary {
    let fallback_id = pack_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

    let manifest_path = pack_path.join("manifest.yaml");
    let Ok(content) = std::fs::read_to_string(&manifest_path) else {
        return ManifestSummary {
            id: fallback_id.clone(),
            name: fallback_id,
            version: String::new(),
            description: None,
        };
    };

    if let Ok(manifest) = serde_yaml::from_str::<melon_scenario::manifest::Manifest>(&content) {
        return ManifestSummary {
            id: manifest.id,
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
        };
    }

    let value = serde_yaml::from_str::<serde_yaml::Value>(&content).ok();
    let field = |key: &str| -> Option<String> {
        value
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_str())
            .map(str::to_string)
    };

    let id = field("id").unwrap_or(fallback_id);
    ManifestSummary {
        name: field("name").unwrap_or_else(|| id.clone()),
        version: field("version").unwrap_or_default(),
        description: field("description"),
        id,
    }
}
