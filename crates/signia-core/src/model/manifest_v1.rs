//! SIGNIA Manifest v1 model.
//!
//! The manifest binds together one or more schema artifacts, compiler inputs,
//! outputs, plugins, and execution constraints into a single deterministic unit.
//!
//! A manifest is:
//! - content-addressable
//! - reproducible
//! - safe to verify independently of execution
//!
//! This is a wire-level model. Do not introduce breaking changes for v1.

#[cfg(feature = "canonical-json")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

/// A SIGNIA manifest instance.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct ManifestV1 {
    /// Manifest version. Must be "v1".
    pub version: String,

    /// Manifest name.
    pub name: String,

    /// Optional human-readable description.
    #[cfg_attr(feature = "canonical-json", serde(default))]
    pub description: Option<String>,

    /// Schema references included in this manifest.
    pub schemas: Vec<SchemaRefV1>,

    /// Compiler inputs.
    pub inputs: Vec<InputRefV1>,

    /// Declared outputs.
    pub outputs: Vec<OutputRefV1>,

    /// Plugins used during compilation.
    #[cfg_attr(feature = "canonical-json", serde(default))]
    pub plugins: Vec<PluginRefV1>,

    /// Execution and resource limits.
    pub limits: LimitsV1,

    /// Arbitrary deterministic labels.
    #[cfg_attr(feature = "canonical-json", serde(default))]
    pub labels: Option<std::collections::BTreeMap<String, String>>,
}

/// Reference to a schema artifact.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct SchemaRefV1 {
    pub name: String,
    pub digest: String,
}

/// Reference to a compiler input.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct InputRefV1 {
    pub r#type: String,
    pub locator: String,
    #[cfg_attr(feature = "canonical-json", serde(default))]
    pub digest: Option<String>,
}

/// Reference to a compiler output.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct OutputRefV1 {
    pub r#type: String,
    pub locator: String,
    #[cfg_attr(feature = "canonical-json", serde(default))]
    pub expected_digest: Option<String>,
}

/// Reference to a plugin.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct PluginRefV1 {
    pub name: String,
    pub version: String,
    #[cfg_attr(feature = "canonical-json", serde(default))]
    pub config: Option<Value>,
}

/// Execution and resource limits.
#[cfg_attr(feature = "canonical-json", derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "canonical-json", serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct LimitsV1 {
    pub max_files: u64,
    pub max_bytes: u64,
    pub max_nodes: u64,
    pub max_edges: u64,
    pub timeout_ms: u64,
    pub network: String,
}

impl ManifestV1 {
    /// Create a new manifest with empty collections.
    pub fn new(name: impl Into<String>, limits: LimitsV1) -> Self {
        Self {
            version: "v1".to_string(),
            name: name.into(),
            description: None,
            schemas: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            plugins: Vec::new(),
            limits,
            labels: None,
        }
    }

    pub fn add_schema(&mut self, s: SchemaRefV1) {
        self.schemas.push(s);
    }

    pub fn add_input(&mut self, i: InputRefV1) {
        self.inputs.push(i);
    }

    pub fn add_output(&mut self, o: OutputRefV1) {
        self.outputs.push(o);
    }

    pub fn add_plugin(&mut self, p: PluginRefV1) {
        self.plugins.push(p);
    }
}

#[cfg(test)]
#[cfg(feature = "canonical-json")]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip() {
        let limits = LimitsV1 {
            max_files: 100,
            max_bytes: 10_000,
            max_nodes: 1_000,
            max_edges: 2_000,
            timeout_ms: 5_000,
            network: "deny".to_string(),
        };

        let mut m = ManifestV1::new("demo", limits);
        m.description = Some("test".to_string());

        m.add_schema(SchemaRefV1 {
            name: "repo".to_string(),
            digest: "a".repeat(64),
        });

        let s = serde_json::to_string(&m).unwrap();
        let back: ManifestV1 = serde_json::from_str(&s).unwrap();
        assert_eq!(back.version, "v1");
        assert_eq!(back.schemas.len(), 1);
    }
}
