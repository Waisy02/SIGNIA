use axum::Router;

use crate::middleware;
use crate::routes;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let router = Router::new()
        .merge(routes::router())
        .with_state(state);

    middleware::wrap(router)
}
