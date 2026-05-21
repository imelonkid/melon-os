use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::Serialize;

use crate::{routes::packs::resolve_pack_dir_by_id, AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/packs/{pack_id}/validate", post(validate_pack))
}

#[derive(Serialize)]
struct ValidationResponse {
    valid: bool,
    errors: Vec<String>,
}

async fn validate_pack(
    State(state): State<AppState>,
    Path(pack_id): Path<String>,
) -> Result<Json<ValidationResponse>, StatusCode> {
    let pack_dir = resolve_pack_dir(&state.scenarios_dir, &pack_id)?;
    let errors = melon_scenario::validation::validate_pack(&pack_dir)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ValidationResponse {
        valid: errors.is_empty(),
        errors,
    }))
}

fn resolve_pack_dir(
    scenarios_dir: &std::path::PathBuf,
    pack_id: &str,
) -> Result<std::path::PathBuf, StatusCode> {
    resolve_pack_dir_by_id(scenarios_dir, pack_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
}
