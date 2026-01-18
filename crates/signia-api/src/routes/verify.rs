use axum::Json;

use crate::dto::requests::VerifyRequest;
use crate::dto::responses::VerifyResponse;
use crate::error::{ApiError, ApiResult};

pub async fn verify(Json(req): Json<VerifyRequest>) -> ApiResult<Json<VerifyResponse>> {
    // Minimal proof verification that is deterministic and useful without chain access.
    // If the caller provides a merkle proof, verify it.
    if let Some(p) = req.merkle_proof.as_ref() {
        let root = hex::decode(&req.root)
            .map_err(|_| ApiError::BadRequest("root must be hex".to_string()))?;
        if root.len() != 32 {
            return Err(ApiError::BadRequest("root must be 32 bytes".to_string()));
        }
        let mut root_arr = [0u8; 32];
        root_arr.copy_from_slice(&root);

        let ok = signia_store::proofs::verify::verify_proof(&req.leaf, &root_arr, p)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;

        return Ok(Json(VerifyResponse { ok, details: if ok { None } else { Some("proof mismatch".to_string()) } }));
    }

    Err(ApiError::BadRequest("missing merkle_proof".to_string()))
}
