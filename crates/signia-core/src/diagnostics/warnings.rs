//! Warning catalog for SIGNIA diagnostics.
//!
//! This module defines a structured set of warning codes and helpers for creating
//! consistent warning diagnostics.
//!
//! Warnings are not fatal by default, but they should be actionable.
//!
//! Determinism constraints:
//! - no machine-specific strings
//! - no timestamps
//! - no random ids

use std::collections::BTreeMap;

use crate::diagnostics::{DiagLevel, Diagnostic};

/// A typed warning code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarningCode(pub &'static str);

impl WarningCode {
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

/// Standard warning codes.
/// Keep this list stable to avoid breaking downstream tooling.
pub mod codes {
    use super::WarningCode;

    pub const NON_CANONICAL_PATH: WarningCode = WarningCode("warn.non_canonical_path");
    pub const NON_UTF8_TEXT: WarningCode = WarningCode("warn.non_utf8_text");
    pub const TRUNCATED_TEXT: WarningCode = WarningCode("warn.truncated_text");
    pub const LARGE_GRAPH: WarningCode = WarningCode("warn.large_graph");
    pub const MISSING_OPTIONAL_FIELD: WarningCode = WarningCode("warn.missing_optional_field");
    pub const UNUSED_PLUGIN: WarningCode = WarningCode("warn.unused_plugin");
    pub const NETWORK_DISABLED: WarningCode = WarningCode("warn.network_disabled");
    pub const LIMIT_NEAR_MAX: WarningCode = WarningCode("warn.limit_near_max");
}

/// Build a warning diagnostic with a code and message.
pub fn warning(code: WarningCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        level: DiagLevel::Warning,
        code: code.as_str().to_string(),
        message: message.into(),
        fields: BTreeMap::new(),
    }
}

/// Warning: input path was normalized.
pub fn non_canonical_path(original: &str, normalized: &str) -> Diagnostic {
    warning(
        codes::NON_CANONICAL_PATH,
        "path was normalized for determinism",
    )
    .with_field("original", original)
    .with_field("normalized", normalized)
}

/// Warning: input text is not valid UTF-8 (caller should provide a summary).
pub fn non_utf8_text(hint: &str) -> Diagnostic {
    warning(
        codes::NON_UTF8_TEXT,
        "text content was not valid UTF-8; treated as binary",
    )
    .with_field("hint", hint)
}

/// Warning: input text was truncated to comply with limits.
pub fn truncated_text(original_bytes: usize, truncated_bytes: usize) -> Diagnostic {
    warning(codes::TRUNCATED_TEXT, "text content was truncated")
        .with_field("originalBytes", original_bytes.to_string())
        .with_field("truncatedBytes", truncated_bytes.to_string())
}

/// Warning: graph is large and may impact compilation.
pub fn large_graph(nodes: usize, edges: usize) -> Diagnostic {
    warning(codes::LARGE_GRAPH, "graph size is large")
        .with_field("nodes", nodes.to_string())
        .with_field("edges", edges.to_string())
}

/// Warning: optional field missing; this is not an error but may reduce UX.
pub fn missing_optional_field(field: &str) -> Diagnostic {
    warning(codes::MISSING_OPTIONAL_FIELD, "optional field is missing").with_field("field", field)
}

/// Warning: plugin declared but not used.
pub fn unused_plugin(name: &str) -> Diagnostic {
    warning(codes::UNUSED_PLUGIN, "plugin declared but not used").with_field("plugin", name)
}

/// Warning: network is disabled by policy.
pub fn network_disabled(policy: &str) -> Diagnostic {
    warning(codes::NETWORK_DISABLED, "network access is disabled by policy")
        .with_field("policy", policy)
}

/// Warning: approaching a limit.
pub fn limit_near_max(limit: &str, used: u64, max: u64) -> Diagnostic {
    warning(codes::LIMIT_NEAR_MAX, "resource usage is near maximum")
        .with_field("limit", limit)
        .with_field("used", used.to_string())
        .with_field("max", max.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warning_has_level_and_code() {
        let d = non_canonical_path("a\\b", "a/b");
        assert_eq!(d.level, DiagLevel::Warning);
        assert_eq!(d.code, "warn.non_canonical_path");
        assert!(d.fields.contains_key("original"));
    }
}
