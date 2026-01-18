//! Dataset schema inference helpers for the built-in `dataset` plugin.
//!
//! This module provides deterministic, best-effort schema inference for common
//! dataset formats using small samples provided by the host.
//!
//! IMPORTANT:
//! - This code performs no filesystem or network I/O.
//! - The host provides file bytes (optionally) for a small sample window.
//! - Inference is deterministic: fixed limits, stable ordering, and stable type rules.
//!
//! Supported formats (best-effort):
//! - JSON Lines (.jsonl, .ndjson)
//! - CSV (.csv)
//!
//! Non-goals:
//! - full validation of all records
//! - supporting every serialization format
//! - streaming huge datasets (host should sample)

#![cfg(feature = "builtin")]

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Maximum number of records to inspect per file.
pub const DEFAULT_MAX_RECORDS: usize = 128;

/// Maximum bytes to scan per file (to avoid large allocations).
pub const DEFAULT_MAX_BYTES: usize = 512 * 1024;

/// A host-provided dataset file sample.
#[derive(Debug, Clone)]
pub struct DatasetFileSample {
    /// Path relative to dataset root.
    pub path: String,
    /// Optional bytes (sample window).
    pub bytes: Option<Vec<u8>>,
}

impl DatasetFileSample {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            bytes: None,
        }
    }

    pub fn with_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.bytes = Some(bytes);
        self
    }
}

/// Simple inferred type system.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarType {
    Null,
    Bool,
    Int,
    Float,
    String,
    Object,
    Array,
}

impl ScalarType {
    fn precedence(&self) -> u8 {
        // Higher wins when merging.
        match self {
            ScalarType::Null => 0,
            ScalarType::Bool => 1,
            ScalarType::Int => 2,
            ScalarType::Float => 3,
            ScalarType::String => 4,
            ScalarType::Array => 5,
            ScalarType::Object => 6,
        }
    }
}

/// A field schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    pub types: BTreeSet<ScalarType>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, FieldSchema>, // for objects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<FieldSchema>>, // for arrays
}

impl FieldSchema {
    pub fn new() -> Self {
        Self {
            types: BTreeSet::new(),
            properties: BTreeMap::new(),
            items: None,
        }
    }

    pub fn with_type(mut self, t: ScalarType) -> Self {
        self.types.insert(t);
        self
    }

    fn merge(&mut self, other: &FieldSchema) {
        self.types.extend(other.types.iter().cloned());

        // Merge object properties deterministically.
        for (k, v) in &other.properties {
            self.properties
                .entry(k.clone())
                .or_insert_with(FieldSchema::new)
                .merge(v);
        }

        // Merge array item schema.
        match (&mut self.items, &other.items) {
            (Some(a), Some(b)) => {
                a.merge(b);
            }
            (None, Some(b)) => {
                self.items = Some(Box::new((**b).clone()));
            }
            _ => {}
        }
    }
}

/// A dataset-wide schema result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetSchema {
    pub files: BTreeMap<String, FileSchema>,
    pub summary: SchemaSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaSummary {
    pub files_scanned: u64,
    pub records_scanned: u64,
    pub fields_observed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSchema {
    pub format: String,
    pub record_schema: FieldSchema,
    pub records_scanned: u64,
}

impl DatasetSchema {
    pub fn empty() -> Self {
        Self {
            files: BTreeMap::new(),
            summary: SchemaSummary {
                files_scanned: 0,
                records_scanned: 0,
                fields_observed: 0,
            },
        }
    }
}

/// Infer a dataset schema from file samples.
pub fn infer_dataset_schema(files: &[DatasetFileSample]) -> Result<DatasetSchema> {
    let mut out = DatasetSchema::empty();

    // Deterministic ordering by path.
    let mut paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    paths.sort();
    paths.dedup();

    let mut records_total = 0u64;
    let mut fields_total = 0u64;
    let mut files_scanned = 0u64;

    for path in paths {
        let sample = files.iter().find(|f| f.path == path).unwrap();
        let Some(bytes) = &sample.bytes else {
            continue;
        };

        files_scanned += 1;

        let bytes = if bytes.len() > DEFAULT_MAX_BYTES {
            &bytes[..DEFAULT_MAX_BYTES]
        } else {
            bytes.as_slice()
        };

        let lower = path.to_ascii_lowercase();
        let (format, schema, recs, fields) = if lower.ends_with(".jsonl") || lower.ends_with(".ndjson") {
            let (s, r, f) = infer_jsonl(bytes)?;
            ("jsonl".to_string(), s, r, f)
        } else if lower.ends_with(".csv") {
            let (s, r, f) = infer_csv(bytes)?;
            ("csv".to_string(), s, r, f)
        } else {
            // Unsupported; skip deterministically.
            continue;
        };

        records_total += recs;
        fields_total += fields;

        out.files.insert(
            path.clone(),
            FileSchema {
                format,
                record_schema: schema,
                records_scanned: recs,
            },
        );
    }

    out.summary = SchemaSummary {
        files_scanned,
        records_scanned: records_total,
        fields_observed: fields_total,
    };

    Ok(out)
}

fn infer_jsonl(bytes: &[u8]) -> Result<(FieldSchema, u64, u64)> {
    let text = std::str::from_utf8(bytes).map_err(|_| anyhow!("jsonl sample is not utf-8"))?;
    let mut schema = FieldSchema::new().with_type(ScalarType::Object);

    let mut records = 0u64;
    let mut fields = 0u64;

    for line in text.lines().take(DEFAULT_MAX_RECORDS) {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(l)
            .map_err(|e| anyhow!("jsonl parse error: {e}"))?;

        let fs = schema_from_json_value(&v);
        fields = fields.saturating_add(count_fields(&fs) as u64);
        schema.merge(&fs);
        records += 1;
    }

    Ok((schema, records, fields))
}

fn infer_csv(bytes: &[u8]) -> Result<(FieldSchema, u64, u64)> {
    let text = std::str::from_utf8(bytes).map_err(|_| anyhow!("csv sample is not utf-8"))?;

    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow!("csv sample has no header"))?;
    let cols: Vec<String> = header.split(',').map(|s| s.trim().to_string()).collect();
    if cols.is_empty() || cols.iter().any(|c| c.is_empty()) {
        return Err(anyhow!("csv header invalid"));
    }

    let mut col_schema: Vec<FieldSchema> = cols.iter().map(|_| FieldSchema::new()).collect();

    let mut records = 0u64;
    for line in lines.take(DEFAULT_MAX_RECORDS) {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        let parts: Vec<&str> = l.split(',').collect();
        for (i, cell) in parts.iter().enumerate().take(cols.len()) {
            let t = infer_scalar_from_str(cell.trim());
            col_schema[i].types.insert(t);
        }
        records += 1;
    }

    let mut record = FieldSchema::new().with_type(ScalarType::Object);
    let mut fields = 0u64;

    for (i, name) in cols.iter().enumerate() {
        let mut fs = col_schema[i].clone();
        // If no observed types, treat as string.
        if fs.types.is_empty() {
            fs.types.insert(ScalarType::String);
        } else {
            // Normalize numeric: if both int and float observed, keep only float.
            if fs.types.contains(&ScalarType::Int) && fs.types.contains(&ScalarType::Float) {
                fs.types.remove(&ScalarType::Int);
            }
        }
        record.properties.insert(name.clone(), fs);
        fields += 1;
    }

    Ok((record, records, fields))
}

fn infer_scalar_from_str(s: &str) -> ScalarType {
    if s.is_empty() || s.eq_ignore_ascii_case("null") {
        return ScalarType::Null;
    }
    if s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("false") {
        return ScalarType::Bool;
    }
    // int?
    if s.parse::<i64>().is_ok() {
        return ScalarType::Int;
    }
    // float?
    if s.parse::<f64>().is_ok() {
        return ScalarType::Float;
    }
    ScalarType::String
}

fn schema_from_json_value(v: &serde_json::Value) -> FieldSchema {
    match v {
        serde_json::Value::Null => FieldSchema::new().with_type(ScalarType::Null),
        serde_json::Value::Bool(_) => FieldSchema::new().with_type(ScalarType::Bool),
        serde_json::Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                FieldSchema::new().with_type(ScalarType::Int)
            } else {
                FieldSchema::new().with_type(ScalarType::Float)
            }
        }
        serde_json::Value::String(_) => FieldSchema::new().with_type(ScalarType::String),
        serde_json::Value::Array(arr) => {
            let mut fs = FieldSchema::new().with_type(ScalarType::Array);
            let mut item = FieldSchema::new();
            for it in arr.iter().take(64) {
                item.merge(&schema_from_json_value(it));
            }
            if item.types.is_empty() {
                item.types.insert(ScalarType::Null);
            }
            fs.items = Some(Box::new(item));
            fs
        }
        serde_json::Value::Object(obj) => {
            let mut fs = FieldSchema::new().with_type(ScalarType::Object);
            for (k, v2) in obj {
                fs.properties.insert(k.clone(), schema_from_json_value(v2));
            }
            fs
        }
    }
}

fn count_fields(fs: &FieldSchema) -> usize {
    // Count leaf properties deterministically.
    if !fs.properties.is_empty() {
        fs.properties.values().map(count_fields).sum()
    } else if let Some(items) = &fs.items {
        count_fields(items)
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_jsonl_schema() {
        let bytes = br#"{"a":1,"b":"x"}
{"a":2,"b":"y","c":true}
"#.to_vec();

        let files = vec![DatasetFileSample::new("train.jsonl").with_bytes(bytes)];
        let schema = infer_dataset_schema(&files).unwrap();
        assert_eq!(schema.summary.files_scanned, 1);
        let f = schema.files.get("train.jsonl").unwrap();
        assert_eq!(f.format, "jsonl");
        assert!(f.record_schema.properties.contains_key("a"));
        assert!(f.record_schema.properties.contains_key("b"));
        assert!(f.record_schema.properties.contains_key("c"));
    }

    #[test]
    fn infers_csv_schema() {
        let bytes = b"a,b,c\n1,2,true\n3,4,false\n".to_vec();
        let files = vec![DatasetFileSample::new("x.csv").with_bytes(bytes)];
        let schema = infer_dataset_schema(&files).unwrap();
        let f = schema.files.get("x.csv").unwrap();
        assert_eq!(f.format, "csv");
        assert!(f.record_schema.properties.contains_key("a"));
    }
}
