use axum::extract::{Path, State};
use axum::http::{header, HeaderMap};
use axum::response::IntoResponse;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub async fn get_artifact(Path(id): Path<String>, State(state): State<AppState>) -> ApiResult<impl IntoResponse> {
    let Some(bytes) = state.store.get_object_bytes(&id).map_err(|e| ApiError::Internal(e.to_string()))? else {
        return Err(ApiError::NotFound);
    };

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/octet-stream".parse().unwrap());
    headers.insert(header::CACHE_CONTROL, "public, max-age=31536000, immutable".parse().unwrap());
    Ok((headers, bytes))
}
