use axum::extract::State;
use axum::Json;

use crate::dto::requests::CompileRequest;
use crate::dto::responses::CompileResponse;
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

use sha2::{Digest, Sha256};

pub async fn compile(State(state): State<AppState>, Json(req): Json<CompileRequest>) -> ApiResult<Json<CompileResponse>> {
    // 1) Canonicalize input JSON deterministically
    let canonical = signia_core::determinism::canonical_json::canonicalize_json(&req.input)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // 2) Detect kind (or respect hint)
    let detected = match req.kind.as_deref() {
        Some("repo") => signia_plugins::builtin::config::schema_detect::DetectedKind::Repo,
        Some("dataset") => signia_plugins::builtin::config::schema_detect::DetectedKind::Dataset,
        Some("workflow") => signia_plugins::builtin::config::schema_detect::DetectedKind::Workflow,
        Some("openapi") => signia_plugins::builtin::config::schema_detect::DetectedKind::OpenApi,
        Some(_) => return Err(ApiError::BadRequest("unknown kind".to_string())),
        None => signia_plugins::builtin::config::schema_detect::detect_input_kind(&canonical)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?
            .kind,
    };

    // 3) Compile via plugin into IR (schema-like JSON) and metadata.
    let mut ctx = signia_core::pipeline::context::PipelineContext::new(
        signia_core::pipeline::context::PipelineConfig::default(),
    );

    let input_key = match detected {
        signia_plugins::builtin::config::schema_detect::DetectedKind::Repo => "repo",
        signia_plugins::builtin::config::schema_detect::DetectedKind::Dataset => "dataset",
        signia_plugins::builtin::config::schema_detect::DetectedKind::Workflow => "workflow",
        signia_plugins::builtin::config::schema_detect::DetectedKind::OpenApi => "openapi",
        signia_plugins::builtin::config::schema_detect::DetectedKind::Unknown => {
            return Err(ApiError::BadRequest("unable to detect input kind".to_string()))
        }
    };
    ctx.inputs.insert(input_key.to_string(), canonical.clone());

    let plugin_id = match detected {
        signia_plugins::builtin::config::schema_detect::DetectedKind::Repo => "builtin.repo",
        signia_plugins::builtin::config::schema_detect::DetectedKind::Dataset => "builtin.dataset",
        signia_plugins::builtin::config::schema_detect::DetectedKind::Workflow => "builtin.workflow",
        signia_plugins::builtin::config::schema_detect::DetectedKind::OpenApi => "builtin.api.openapi",
        signia_plugins::builtin::config::schema_detect::DetectedKind::Unknown => "",
    };

    let plugin = state.plugins.get(plugin_id).ok_or_else(|| ApiError::Internal(format!("plugin not found: {plugin_id}")))?;
    plugin
        .execute(&signia_plugins::plugin::PluginInput::Pipeline(&mut ctx))
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let ir_value = serde_json::to_value(&ctx.ir).map_err(|e| ApiError::Internal(e.to_string()))?;
    let schema_json = signia_core::determinism::canonical_json::canonicalize_json(&ir_value)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // 4) Create manifest/proof (deterministic hashes)
    let schema_bytes = serde_json::to_vec(&schema_json).map_err(|e| ApiError::Internal(e.to_string()))?;
    let schema_id = state.store.put_object_bytes(&schema_bytes).map_err(|e| ApiError::Internal(e.to_string()))?;

    let manifest = build_manifest(&canonical, &schema_id, input_key);
    let manifest_bytes = serde_json::to_vec(&manifest).map_err(|e| ApiError::Internal(e.to_string()))?;
    let manifest_id = state.store.put_object_bytes(&manifest_bytes).map_err(|e| ApiError::Internal(e.to_string()))?;

    let proof = build_proof(&canonical, &schema_id, &manifest_id);
    let proof_bytes = serde_json::to_vec(&proof).map_err(|e| ApiError::Internal(e.to_string()))?;
    let proof_id = state.store.put_object_bytes(&proof_bytes).map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(CompileResponse {
        kind: input_key.to_string(),
        schema_id,
        manifest_id,
        proof_id,
        metadata: ctx.metadata,
    }))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

fn build_manifest(input: &serde_json::Value, schema_id: &str, input_key: &str) -> serde_json::Value {
    let input_bytes = serde_json::to_vec(input).unwrap_or_default();
    serde_json::json!({
        "version": "v1",
        "inputKind": input_key,
        "inputHash": sha256_hex(&input_bytes),
        "schemaObjectId": schema_id,
        "createdAt": time::OffsetDateTime::now_utc().unix_timestamp(),
    })
}

fn build_proof(input: &serde_json::Value, schema_id: &str, manifest_id: &str) -> serde_json::Value {
    let input_bytes = serde_json::to_vec(input).unwrap_or_default();
    let leaf = sha256_hex(&input_bytes);

    // Proof here is a simple two-leaf Merkle tree: [inputHash, schemaIdHash]
    let schema_leaf = sha256_hex(schema_id.as_bytes());
    let leaves = vec![leaf.clone(), schema_leaf.clone()];
    let root = signia_store::proofs::merkle::merkle_root_hex(&leaves).unwrap_or_else(|_| "".to_string());
    let proof0 = signia_store::proofs::merkle::merkle_proof(&leaves, 0).ok();

    serde_json::json!({
        "version": "v1",
        "root": root,
        "leaf": leaf,
        "schemaLeaf": schema_leaf,
        "manifestObjectId": manifest_id,
        "merkleProof": proof0,
    })
}
