//! Built-in `dataset` plugin for SIGNIA.
//!
//! This plugin handles dataset-like inputs that have already been materialized
//! by the host into a deterministic structure.
//!
//! Responsibilities:
//! - validate dataset metadata and files list
//! - build a deterministic IR that represents dataset structure
//! - compute a stable dataset fingerprint (hash)
//! - attach metadata to PipelineContext for downstream compilation
//!
//! Non-responsibilities:
//! - downloading datasets
//! - decompressing archives
//! - parsing proprietary formats
//!
//! All raw I/O must be done by the host.

#![cfg(feature = "builtin")]

use anyhow::Result;
use serde_json::Value;

use signia_core::determinism::hashing::hash_bytes_hex;
use signia_core::model::ir::{IrEdge, IrGraph, IrNode};
use signia_core::pipeline::context::PipelineContext;

use crate::plugin::{Plugin, PluginInput, PluginOutput};
use crate::registry::PluginRegistry;
use crate::spec::PluginSpec;

/// Register the dataset plugin.
pub fn register(registry: &mut PluginRegistry) {
    let spec = PluginSpec::new("builtin.dataset", "Dataset Plugin", "0.1.0")
        .support("dataset")
        .limit("max_nodes", 300_000)
        .limit("max_edges", 600_000)
        .want("network", false)
        .want("filesystem", false)
        .meta("category", "data");

    registry
        .register(spec, Box::new(DatasetPlugin))
        .expect("failed to register builtin.dataset");
}

/// Dataset plugin implementation.
pub struct DatasetPlugin;

impl Plugin for DatasetPlugin {
    fn name(&self) -> &str {
        "dataset"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn supports(&self, input_type: &str) -> bool {
        input_type == "dataset"
    }

    fn execute(&self, input: &PluginInput) -> Result<PluginOutput> {
        let ctx = match input {
            PluginInput::Pipeline(ctx) => ctx,
            _ => anyhow::bail!("dataset plugin requires pipeline input"),
        };

        execute_dataset(ctx)?;
        Ok(PluginOutput::None)
    }
}

fn execute_dataset(ctx: &mut PipelineContext) -> Result<()> {
    let meta = ctx
        .inputs
        .get("dataset")
        .ok_or_else(|| anyhow::anyhow!("missing dataset input"))?;

    let name = get_str(meta, "name")?;
    let version = meta.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
    let files = meta
        .get("files")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("dataset.files missing or invalid"))?;

    let mut graph = IrGraph::new();

    let root = IrNode::new("dataset", name);
    let root_id = graph.add_node(root);

    // Add version node
    let ver_node = IrNode::new("version", version);
    let ver_id = graph.add_node(ver_node);
    graph.add_edge(IrEdge::new(root_id, ver_id, "version"));

    // Add file nodes
    for f in files {
        let path = get_str(f, "path")?;
        let size = f.get("size").and_then(|v| v.as_u64()).unwrap_or(0);

        let node = IrNode::new("file", path);
        let file_id = graph.add_node(node);
        graph.add_edge(IrEdge::new(root_id, file_id, "contains"));

        // Attach size as a scalar node (keeps IR simple and deterministic)
        let size_node = IrNode::new("size", size.to_string());
        let size_id = graph.add_node(size_node);
        graph.add_edge(IrEdge::new(file_id, size_id, "has"));
    }

    // Compute a stable dataset fingerprint:
    // path \t size \n for each file sorted by path
    let mut entries: Vec<(String, u64)> = Vec::new();
    for f in files {
        let p = get_str(f, "path")?.to_string();
        let s = f.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
        entries.push((p, s));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut buf = Vec::new();
    for (p, s) in entries {
        buf.extend_from_slice(p.as_bytes());
        buf.extend_from_slice(b"\t");
        buf.extend_from_slice(s.to_string().as_bytes());
        buf.extend_from_slice(b"\n");
    }
    let fingerprint = hash_bytes_hex(&buf)?;

    ctx.metadata
        .insert("datasetFingerprint".to_string(), Value::String(fingerprint));

    ctx.ir = Some(graph);
    Ok(())
}

fn get_str<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(|x| x.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing or invalid string field: {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use signia_core::pipeline::context::PipelineConfig;

    #[test]
    fn dataset_plugin_executes() {
        let mut ctx = PipelineContext::new(PipelineConfig::default());
        ctx.inputs.insert(
            "dataset".to_string(),
            json!({
                "name": "my-dataset",
                "version": "v1",
                "files": [
                    { "path": "train.jsonl", "size": 10 },
                    { "path": "test.jsonl", "size": 5 }
                ]
            }),
        );

        let plugin = DatasetPlugin;
        plugin.execute(&PluginInput::Pipeline(&mut ctx)).unwrap();

        assert!(ctx.ir.is_some());
        assert!(ctx.metadata.get("datasetFingerprint").is_some());
    }
}
