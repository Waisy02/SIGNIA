use axum::http::{HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

pub fn layer() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any)
}
