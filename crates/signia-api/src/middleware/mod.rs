use axum::Router;

mod auth;
mod cors;
mod rate_limit;
mod request_id;

pub fn wrap(router: Router) -> Router {
    router
        .layer(request_id::layer())
        .layer(rate_limit::layer())
        .layer(cors::layer())
        .layer(auth::layer())
}
