use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

use crate::error::ApiError;
use crate::state::AppState;

pub fn layer() -> tower::layer::util::Identity {
    // Auth is implemented as a route-level middleware via `axum::middleware::from_fn_with_state`
    // in `routes/mod.rs`, but this layer hook is kept for future expansion.
    tower::layer::util::Identity::new()
}

pub async fn enforce(State(state): State<AppState>, req: Request<axum::body::Body>, next: Next) -> Result<Response, ApiError> {
    let mode = state.cfg.auth.mode.as_str();
    if mode == "disabled" {
        return Ok(next.run(req).await);
    }

    // Extract bearer token.
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    match (mode, token) {
        ("required", None) => Err(ApiError::Unauthorized),
        ("optional", None) => Ok(next.run(req).await),
        (_, Some(t)) => {
            if state.cfg.auth.bearer_tokens.is_empty() {
                // If no tokens configured, accept any token in optional mode, reject in required mode.
                if mode == "required" {
                    return Err(ApiError::Forbidden);
                }
                return Ok(next.run(req).await);
            }
            if state.cfg.auth.bearer_tokens.iter().any(|x| x == &t) {
                Ok(next.run(req).await)
            } else {
                Err(ApiError::Forbidden)
            }
        }
        _ => Ok(next.run(req).await),
    }
}
