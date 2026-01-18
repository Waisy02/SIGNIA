use axum::routing::get;
use axum::Json;
use axum::Router;
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct RegistryStatus {
    pub enabled: bool,
    pub note: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/status", get(status))
}

pub async fn status() -> Json<RegistryStatus> {
    Json(RegistryStatus {
        enabled: false,
        note: "On-chain registry integration is provided by signia-program and host wiring".to_string(),
    })
}
