//! Hint diagnostics for SIGNIA.
//!
//! Hints are informational diagnostics intended to guide users toward
//! better usage patterns, performance improvements, or best practices.
//!
//! Hints must:
//! - never change compilation outcome
//! - never include machine-specific data
//! - remain deterministic for identical inputs
//!
//! This module complements `warnings.rs` but represents lower-severity guidance.

use std::collections::BTreeMap;

use crate::diagnostics::{DiagLevel, Diagnostic};

/// A typed hint code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HintCode(pub &'static str);

impl HintCode {
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

/// Standard hint codes.
/// These are stable identifiers for tooling and UI layers.
pub mod codes {
    use super::HintCode;

    pub const USE_EXPLICIT_VERSION: HintCode = HintCode("hint.use_explicit_version");
    pub const ENABLE_REPRODUCIBLE: HintCode = HintCode("hint.enable_reproducible");
    pub const SPLIT_LARGE_INPUT: HintCode = HintCode("hint.split_large_input");
    pub const PIN_DEPENDENCIES: HintCode = HintCode("hint.pin_dependencies");
    pub const CACHE_RESULTS: HintCode = HintCode("hint.cache_results");
    pub const DECLARE_LIMITS: HintCode = HintCode("hint.declare_limits");
}

/// Build a hint diagnostic.
pub fn hint(code: HintCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        level: DiagLevel::Info,
        code: code.as_str().to_string(),
        message: message.into(),
        fields: BTreeMap::new(),
    }
}

/// Hint: recommend explicit versioning.
pub fn use_explicit_version(entity: &str) -> Diagnostic {
    hint(
        codes::USE_EXPLICIT_VERSION,
        "explicit versioning is recommended for deterministic builds",
    )
    .with_field("entity", entity)
}

/// Hint: recommend enabling reproducible mode.
pub fn enable_reproducible() -> Diagnostic {
    hint(
        codes::ENABLE_REPRODUCIBLE,
        "enable reproducible mode to improve determinism guarantees",
    )
}

/// Hint: recommend splitting large inputs.
pub fn split_large_input(kind: &str, size: usize) -> Diagnostic {
    hint(
        codes::SPLIT_LARGE_INPUT,
        "consider splitting large input into smaller units",
    )
    .with_field("kind", kind)
    .with_field("size", size.to_string())
}

/// Hint: recommend pinning dependencies.
pub fn pin_dependencies(dep: &str) -> Diagnostic {
    hint(
        codes::PIN_DEPENDENCIES,
        "pin dependency versions to avoid nondeterministic resolution",
    )
    .with_field("dependency", dep)
}

/// Hint: recommend caching results.
pub fn cache_results(stage: &str) -> Diagnostic {
    hint(
        codes::CACHE_RESULTS,
        "caching intermediate results may improve performance",
    )
    .with_field("stage", stage)
}

/// Hint: recommend declaring limits explicitly.
pub fn declare_limits(limit: &str) -> Diagnostic {
    hint(
        codes::DECLARE_LIMITS,
        "explicitly declaring limits improves safety and predictability",
    )
    .with_field("limit", limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_is_info_level() {
        let d = enable_reproducible();
        assert_eq!(d.level, DiagLevel::Info);
        assert!(d.code.starts_with("hint."));
    }

    #[test]
    fn hint_has_fields() {
        let d = split_large_input("dataset", 1024);
        assert_eq!(d.fields.get("kind").unwrap(), "dataset");
    }
}
