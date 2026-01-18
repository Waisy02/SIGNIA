use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResponse {
    pub kind: String,
    pub schema_id: String,
    pub manifest_id: String,
    pub proof_id: String,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub ok: bool,
    #[serde(default)]
    pub details: Option<String>,
}
