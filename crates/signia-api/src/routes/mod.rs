use axum::routing::{get, post};
use axum::Router;

use crate::middleware::{auth, rate_limit};
use crate::state::AppState;

mod artifacts;
mod compile;
mod health;
mod plugins;
mod registry;
mod verify;

pub fn router() -> Router<AppState> {
    let v1 = Router::new()
        .route("/compile", post(compile::compile))
        .route("/verify", post(verify::verify))
        .route("/artifacts/:id", get(artifacts::get_artifact))
        .route("/plugins", get(plugins::list_plugins))
        .nest("/registry", registry::router());

    Router::new()
        .route("/healthz", get(health::healthz))
        .nest("/v1", v1)
        .layer(axum::middleware::from_fn_with_state(
            AppState::clone,
            rate_limit::enforce,
        ))
        .layer(axum::middleware::from_fn_with_state(
            AppState::clone,
            auth::enforce,
        ))
}
