use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/packs/{pack_id}/files", get(list_pack_files))
        .route(
            "/api/packs/{pack_id}/files/{*file_path}",
            get(read_pack_file),
        )
        .route(
            "/api/packs/{pack_id}/files/{*file_path}",
            put(save_pack_file),
        )
}

#[derive(Serialize)]
struct FileInfo {
    name: String,
    path: String,
}

async fn list_pack_files(
    State(state): State<AppState>,
    AxumPath(pack_id): AxumPath<String>,
) -> Result<Json<Vec<FileInfo>>, StatusCode> {
    let pack_dir = resolve_pack_dir(&state.scenarios_dir, &pack_id)?;

    let mut files = Vec::new();
    collect_files(&pack_dir, &pack_dir, &mut files)?;

    Ok(Json(files))
}

async fn read_pack_file(
    State(state): State<AppState>,
    AxumPath((pack_id, file_path)): AxumPath<(String, String)>,
) -> Result<Json<FileResponse>, StatusCode> {
    let pack_dir = resolve_pack_dir(&state.scenarios_dir, &pack_id)?;
    let full_path = safe_join(&pack_dir, &file_path)?;

    let content = tokio::fs::read_to_string(&full_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(FileResponse { content }))
}

#[derive(Deserialize)]
struct FileSaveRequest {
    content: String,
}

#[derive(Serialize)]
struct FileResponse {
    content: String,
}

async fn save_pack_file(
    State(state): State<AppState>,
    AxumPath((pack_id, file_path)): AxumPath<(String, String)>,
    Json(req): Json<FileSaveRequest>,
) -> Result<StatusCode, StatusCode> {
    let pack_dir = resolve_pack_dir(&state.scenarios_dir, &pack_id)?;
    let full_path = safe_join(&pack_dir, &file_path)?;

    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    tokio::fs::write(&full_path, req.content)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

fn resolve_pack_dir(scenarios_dir: &PathBuf, pack_id: &str) -> Result<PathBuf, StatusCode> {
    let entries = std::fs::read_dir(scenarios_dir).map_err(|_| StatusCode::NOT_FOUND)?;

    for entry in entries {
        let entry = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if entry.path().is_dir() {
            let manifest_path = entry.path().join("manifest.yaml");
            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(manifest) =
                    serde_yaml::from_str::<melon_scenario::manifest::Manifest>(&content)
                {
                    if manifest.id == pack_id {
                        return Ok(entry.path());
                    }
                }
            }
        }
    }

    Err(StatusCode::NOT_FOUND)
}

fn safe_join(base: &Path, relative: &str) -> Result<PathBuf, StatusCode> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute() {
        return Err(StatusCode::FORBIDDEN);
    }

    for component in relative_path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

    let full_path = base.join(relative_path);

    // Prevent path traversal
    let canonical_base = base.canonicalize().map_err(|_| StatusCode::NOT_FOUND)?;
    let canonical_parent = full_path
        .parent()
        .ok_or(StatusCode::FORBIDDEN)?
        .canonicalize()
        .unwrap_or_else(|_| full_path.parent().unwrap().to_path_buf());

    if !canonical_parent.starts_with(&canonical_base) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(full_path)
}

fn collect_files(
    dir: &PathBuf,
    base: &PathBuf,
    files: &mut Vec<FileInfo>,
) -> Result<(), StatusCode> {
    let entries = std::fs::read_dir(dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for entry in entries {
        let entry = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let path = entry.path();

        if path.is_dir() {
            collect_files(&path, base, files)?;
        } else if path.extension().map_or(false, |ext| {
            ext == "yaml" || ext == "yml" || ext == "md" || ext == "json"
        }) {
            let rel_path = path
                .strip_prefix(base)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            files.push(FileInfo {
                name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                path: rel_path,
            });
        }
    }

    Ok(())
}
