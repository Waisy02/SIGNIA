use serde::{Deserialize, Serialize};

use signia_store::proofs::merkle::MerkleProof;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompileRequest {
    /// Optional hint: repo|dataset|workflow|openapi
    #[serde(default)]
    pub kind: Option<String>,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerifyRequest {
    pub root: String,
    pub leaf: String,
    #[serde(default)]
    pub merkle_proof: Option<MerkleProof>,
}
