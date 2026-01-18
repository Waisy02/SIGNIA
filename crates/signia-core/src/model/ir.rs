//! Intermediate Representation (IR) for SIGNIA compilation.
//!
//! The SIGNIA wire formats (Schema/Manifest/Proof) are versioned and designed for
//! interoperability. During compilation, however, it is often convenient to use an
//! internal IR that is:
//! - richer for compiler authors (plugins)
//! - explicit about provenance (where a node came from)
//! - easy to validate and normalize before emitting the stable v1 schema
//!
//! This IR is used by plugins and the compiler pipeline. It is not intended to be a
//! stable external API. Still, it is deterministic-friendly: it avoids non-deterministic
//! fields and provides explicit ordering strategies.
//!
//! Key differences vs SchemaV1:
//! - IR nodes/edges can carry typed provenance and diagnostics.
//! - IR supports staged normalization (paths, ids, attributes).
//! - IR supports optional source spans for UX (Console), but these spans must never
//!   participate in canonical hashing unless explicitly included.
//!
//! Recommended usage:
//! - plugins produce `IrGraph`
//! - compiler normalizes + validates
//! - compiler emits `SchemaV1` with stable ids and deterministic attributes

use std::collections::{BTreeMap, BTreeSet};

use crate::errors::{SigniaError, SigniaResult};
use crate::model::v1::{EdgeV1, EntityV1, SchemaV1};

/// Canonical string identifier for IR nodes and edges.
///
/// In IR, ids may be temporary. The compiler will assign final stable ids during
/// emission. Still, IR ids should be predictable whenever possible.
pub type IrId = String;

/// A stable-ish key used for ordering and deduplicating IR nodes.
///
/// Plugins should aim to produce stable `key`s because they map cleanly to final ids.
pub type IrKey = String;

/// A single IR attribute value.
///
/// We intentionally keep this compatible with JSON types.
#[derive(Debug, Clone, PartialEq)]
pub enum IrValue {
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
    Array(Vec<IrValue>),
    Object(BTreeMap<String, IrValue>),
}

impl IrValue {
    /// Convert IR value into a serde_json::Value for emission.
    #[cfg(feature = "canonical-json")]
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            IrValue::Null => serde_json::Value::Null,
            IrValue::Bool(b) => serde_json::Value::Bool(*b),
            IrValue::I64(n) => serde_json::Value::Number((*n).into()),
            IrValue::F64(x) => {
                let n = serde_json::Number::from_f64(*x).unwrap_or_else(|| serde_json::Number::from(0));
                serde_json::Value::Number(n)
            }
            IrValue::String(s) => serde_json::Value::String(s.clone()),
            IrValue::Array(a) => serde_json::Value::Array(a.iter().map(|v| v.to_json()).collect()),
            IrValue::Object(o) => {
                let mut m = serde_json::Map::new();
                for (k, v) in o.iter() {
                    m.insert(k.clone(), v.to_json());
                }
                serde_json::Value::Object(m)
            }
        }
    }
}

/// Provenance information for an IR node/edge.
///
/// This helps explain "where did this come from" in Console UX.
/// Provenance fields are optional and must never affect canonical hashing unless
/// explicitly included by the compiler.
#[derive(Debug, Clone)]
pub struct Provenance {
    pub source: ProvenanceSource,
    pub hint: Option<String>,
    pub span: Option<SourceSpan>,
}

/// The primary source category of a compiler output item.
#[derive(Debug, Clone)]
pub enum ProvenanceSource {
    FilePath(String),
    Url(String),
    Inline(String),
    Generated(String),
}

/// A loose source span for UX (not for hashing).
#[derive(Debug, Clone)]
pub struct SourceSpan {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// IR node representation.
#[derive(Debug, Clone)]
pub struct IrNode {
    /// Temporary id used within IR graph.
    pub id: IrId,

    /// Stable-ish key for deduplication and final id derivation.
    pub key: IrKey,

    /// Node type (e.g. file, module, endpoint, table).
    pub node_type: String,

    /// Display name.
    pub name: String,

    /// Deterministic attributes.
    pub attrs: BTreeMap<String, IrValue>,

    /// Optional digests (hex) for content-addressing.
    pub digests: Vec<IrDigest>,

    /// Optional provenance.
    pub provenance: Option<Provenance>,

    /// Non-fatal diagnostics produced during compilation.
    pub diagnostics: Vec<Diagnostic>,
}

/// IR edge representation.
#[derive(Debug, Clone)]
pub struct IrEdge {
    pub id: IrId,
    pub key: IrKey,
    pub edge_type: String,
    pub from: IrId,
    pub to: IrId,
    pub attrs: BTreeMap<String, IrValue>,
    pub provenance: Option<Provenance>,
    pub diagnostics: Vec<Diagnostic>,
}

/// A digest attached to an IR node.
#[derive(Debug, Clone)]
pub struct IrDigest {
    pub alg: String, // "sha256" | "blake3"
    pub hex: String, // hex string
}

/// Compiler diagnostic for UX and debugging.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: String,
    pub message: String,
    pub details: BTreeMap<String, IrValue>,
}

#[derive(Debug, Clone, Copy)]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
}

/// IR graph container.
#[derive(Debug, Clone)]
pub struct IrGraph {
    pub nodes: BTreeMap<IrId, IrNode>,
    pub edges: BTreeMap<IrId, IrEdge>,
}

impl IrGraph {
    /// Create an empty graph.
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
        }
    }

    /// Insert a node. Fails if id already exists.
    pub fn insert_node(&mut self, node: IrNode) -> SigniaResult<()> {
        if self.nodes.contains_key(&node.id) {
            return Err(SigniaError::invalid_argument(format!(
                "duplicate IR node id: {}",
                node.id
            )));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    /// Insert an edge. Fails if id already exists.
    pub fn insert_edge(&mut self, edge: IrEdge) -> SigniaResult<()> {
        if self.edges.contains_key(&edge.id) {
            return Err(SigniaError::invalid_argument(format!(
                "duplicate IR edge id: {}",
                edge.id
            )));
        }
        self.edges.insert(edge.id.clone(), edge);
        Ok(())
    }

    /// Basic validation:
    /// - all edge endpoints exist
    /// - stable keys are unique (best-effort)
    pub fn validate_basic(&self) -> SigniaResult<()> {
        let mut node_keys = BTreeSet::new();
        for n in self.nodes.values() {
            if !node_keys.insert(n.key.clone()) {
                return Err(SigniaError::invalid_argument(format!(
                    "duplicate IR node key: {}",
                    n.key
                )));
            }
        }

        let mut edge_keys = BTreeSet::new();
        for e in self.edges.values() {
            if !self.nodes.contains_key(&e.from) {
                return Err(SigniaError::invalid_argument(format!(
                    "edge {} references missing from node id {}",
                    e.id, e.from
                )));
            }
            if !self.nodes.contains_key(&e.to) {
                return Err(SigniaError::invalid_argument(format!(
                    "edge {} references missing to node id {}",
                    e.id, e.to
                )));
            }
            if !edge_keys.insert(e.key.clone()) {
                return Err(SigniaError::invalid_argument(format!(
                    "duplicate IR edge key: {}",
                    e.key
                )));
            }
        }

        Ok(())
    }

    /// Deterministic ordering of nodes for emission (by key, then id).
    pub fn ordered_nodes(&self) -> Vec<&IrNode> {
        let mut v: Vec<&IrNode> = self.nodes.values().collect();
        v.sort_by(|a, b| a.key.cmp(&b.key).then(a.id.cmp(&b.id)));
        v
    }

    /// Deterministic ordering of edges for emission (by key, then id).
    pub fn ordered_edges(&self) -> Vec<&IrEdge> {
        let mut v: Vec<&IrEdge> = self.edges.values().collect();
        v.sort_by(|a, b| a.key.cmp(&b.key).then(a.id.cmp(&b.id)));
        v
    }

    /// Emit a v1 schema from this IR.
    ///
    /// The caller supplies:
    /// - `kind`: schema kind
    /// - `meta`: schema meta section already constructed by higher layer
    /// - `id_strategy`: a mapping that assigns stable final ids from IR keys
    ///
    /// This emission intentionally drops:
    /// - provenance
    /// - diagnostics
    ///
    /// unless the caller chooses to embed them into attrs explicitly.
    #[cfg(feature = "canonical-json")]
    pub fn emit_schema_v1(
        &self,
        kind: &str,
        meta: serde_json::Value,
        id_strategy: &dyn IdStrategy,
    ) -> SigniaResult<SchemaV1> {
        self.validate_basic()?;

        // Map IR node ids -> final entity ids
        let mut ent_id_map: BTreeMap<IrId, String> = BTreeMap::new();
        for n in self.ordered_nodes() {
            let ent_id = id_strategy.entity_id(&n.key, &n.node_type)?;
            ent_id_map.insert(n.id.clone(), ent_id);
        }

        // Build entities
        let mut entities: Vec<EntityV1> = Vec::with_capacity(self.nodes.len());
        for n in self.ordered_nodes() {
            let id = ent_id_map.get(&n.id).expect("missing id map").clone();

            let mut attrs_json = serde_json::Map::new();
            for (k, v) in n.attrs.iter() {
                attrs_json.insert(k.clone(), v.to_json());
            }

            let digests = if n.digests.is_empty() {
                None
            } else {
                Some(
                    n.digests
                        .iter()
                        .map(|d| crate::model::v1::DigestV1 { alg: d.alg.clone(), hex: d.hex.clone() })
                        .collect(),
                )
            };

            entities.push(EntityV1 {
                id,
                r#type: n.node_type.clone(),
                name: n.name.clone(),
                attrs: serde_json::Value::Object(attrs_json),
                digests,
            });
        }

        // Build edges
        let mut edges: Vec<EdgeV1> = Vec::with_capacity(self.edges.len());
        for e in self.ordered_edges() {
            let from = ent_id_map.get(&e.from).ok_or_else(|| {
                SigniaError::invariant(format!("missing from mapping for edge {}", e.id))
            })?;
            let to = ent_id_map.get(&e.to).ok_or_else(|| {
                SigniaError::invariant(format!("missing to mapping for edge {}", e.id))
            })?;

            let edge_id = id_strategy.edge_id(&e.key, &e.edge_type, from, to)?;

            let mut attrs_json = serde_json::Map::new();
            for (k, v) in e.attrs.iter() {
                attrs_json.insert(k.clone(), v.to_json());
            }

            edges.push(EdgeV1 {
                id: edge_id,
                r#type: e.edge_type.clone(),
                from: from.clone(),
                to: to.clone(),
                attrs: serde_json::Value::Object(attrs_json),
            });
        }

        Ok(SchemaV1 {
            version: "v1".to_string(),
            kind: kind.to_string(),
            meta,
            entities,
            edges,
        })
    }
}

/// Strategy for assigning stable final ids.
///
/// The simplest strategy:
/// - derive `ent:<type>:<hash(key)>`
/// - derive `edge:<type>:<hash(from,to,key)>`
///
/// Implementations must be deterministic and stable across machines.
pub trait IdStrategy {
    fn entity_id(&self, key: &str, node_type: &str) -> SigniaResult<String>;
    fn edge_id(&self, key: &str, edge_type: &str, from_ent_id: &str, to_ent_id: &str) -> SigniaResult<String>;
}

/// A default id strategy that uses SHA-256 and short prefixes.
///
/// This is intended for internal use and tests. Production deployments may choose
/// different readable formats (still deterministic).
#[derive(Debug, Clone)]
pub struct DefaultIdStrategy {
    pub prefix_entity: String,
    pub prefix_edge: String,
}

impl Default for DefaultIdStrategy {
    fn default() -> Self {
        Self {
            prefix_entity: "ent".to_string(),
            prefix_edge: "edge".to_string(),
        }
    }
}

impl DefaultIdStrategy {
    fn short_hex(hex64: &str) -> String {
        // Use the first 16 hex chars for readability while keeping collisions unlikely
        hex64.chars().take(16).collect()
    }

    fn sha256_hex(input: &[u8]) -> SigniaResult<String> {
        use crate::hash::{hash_bytes, HashAlg};
        let d = hash_bytes(HashAlg::Sha256, input)?;
        Ok(hex::encode(d))
    }
}

impl IdStrategy for DefaultIdStrategy {
    fn entity_id(&self, key: &str, node_type: &str) -> SigniaResult<String> {
        let payload = format!("node|{node_type}|{key}");
        let hex64 = Self::sha256_hex(payload.as_bytes())?;
        Ok(format!("{}:{}:{}", self.prefix_entity, node_type, Self::short_hex(&hex64)))
    }

    fn edge_id(&self, key: &str, edge_type: &str, from_ent_id: &str, to_ent_id: &str) -> SigniaResult<String> {
        let payload = format!("edge|{edge_type}|{from_ent_id}|{to_ent_id}|{key}");
        let hex64 = Self::sha256_hex(payload.as_bytes())?;
        Ok(format!("{}:{}:{}", self.prefix_edge, edge_type, Self::short_hex(&hex64)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ir_graph_basic_validation() {
        let mut g = IrGraph::new();

        g.insert_node(IrNode {
            id: "n1".to_string(),
            key: "repo:root".to_string(),
            node_type: "repo".to_string(),
            name: "demo".to_string(),
            attrs: BTreeMap::new(),
            digests: vec![],
            provenance: None,
            diagnostics: vec![],
        })
        .unwrap();

        g.insert_node(IrNode {
            id: "n2".to_string(),
            key: "file:readme".to_string(),
            node_type: "file".to_string(),
            name: "README.md".to_string(),
            attrs: BTreeMap::new(),
            digests: vec![],
            provenance: None,
            diagnostics: vec![],
        })
        .unwrap();

        g.insert_edge(IrEdge {
            id: "e1".to_string(),
            key: "contains:root:readme".to_string(),
            edge_type: "contains".to_string(),
            from: "n1".to_string(),
            to: "n2".to_string(),
            attrs: BTreeMap::new(),
            provenance: None,
            diagnostics: vec![],
        })
        .unwrap();

        g.validate_basic().unwrap();
    }

    #[test]
    fn default_id_strategy_is_deterministic() {
        let s = DefaultIdStrategy::default();
        let a = s.entity_id("repo:root", "repo").unwrap();
        let b = s.entity_id("repo:root", "repo").unwrap();
        assert_eq!(a, b);

        let e1 = s.edge_id("k", "contains", &a, "ent:file:x",).unwrap();
        let e2 = s.edge_id("k", "contains", &a, "ent:file:x",).unwrap();
        assert_eq!(e1, e2);
    }
}
