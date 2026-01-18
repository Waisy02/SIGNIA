use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct PluginInfo {
    pub id: String,
    pub version: String,
    pub kind: String,
}

#[derive(Serialize)]
pub struct PluginsResponse {
    pub plugins: Vec<PluginInfo>,
}

pub async fn list_plugins(State(state): State<AppState>) -> Json<PluginsResponse> {
    let mut out = Vec::new();
    for spec in state.plugins.list() {
        out.push(PluginInfo {
            id: spec.id,
            version: spec.version,
            kind: spec.kind,
        });
    }
    Json(PluginsResponse { plugins: out })
}
