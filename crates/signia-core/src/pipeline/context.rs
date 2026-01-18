//! Pipeline execution context for SIGNIA.
//!
//! PipelineContext carries:
//! - deterministic clock input
//! - string and JSON parameters
//! - diagnostics produced by stages
//! - execution-scoped metadata
//!
//! It is explicitly mutable and passed between stages.
//! It must remain serializable and deterministic-friendly.

use std::collections::BTreeMap;

use crate::errors::{SigniaError, SigniaResult};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

/// Deterministic clock input.
///
/// Core never reads system time. Callers must inject timestamps.
#[derive(Debug, Clone)]
pub struct Clock {
    pub now_iso8601: String,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            now_iso8601: "1970-01-01T00:00:00Z".to_string(),
        }
    }
}

/// Diagnostic emitted by pipeline stages.
#[derive(Debug, Clone)]
pub struct PipelineDiagnostic {
    pub level: DiagnosticLevel,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
}

/// Shared pipeline execution context.
#[derive(Debug, Clone)]
pub struct PipelineContext {
    /// Deterministic clock.
    pub clock: Clock,

    /// String parameters.
    pub params: BTreeMap<String, String>,

    /// JSON parameters.
    #[cfg(feature = "canonical-json")]
    pub json_params: BTreeMap<String, Value>,

    /// Collected diagnostics.
    pub diagnostics: Vec<PipelineDiagnostic>,
}

impl Default for PipelineContext {
    fn default() -> Self {
        Self {
            clock: Clock::default(),
            params: BTreeMap::new(),
            #[cfg(feature = "canonical-json")]
            json_params: BTreeMap::new(),
            diagnostics: Vec::new(),
        }
    }
}

impl PipelineContext {
    /// Set a string parameter.
    pub fn set_param(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.params.insert(key.into(), value.into());
    }

    /// Get a string parameter.
    pub fn get_param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    /// Set a JSON parameter.
    #[cfg(feature = "canonical-json")]
    pub fn set_json_param(&mut self, key: impl Into<String>, value: Value) {
        self.json_params.insert(key.into(), value);
    }

    /// Get a JSON parameter.
    #[cfg(feature = "canonical-json")]
    pub fn get_json_param(&self, key: &str) -> Option<&Value> {
        self.json_params.get(key)
    }

    /// Push an info diagnostic.
    pub fn push_info(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.diagnostics.push(PipelineDiagnostic {
            level: DiagnosticLevel::Info,
            code: code.into(),
            message: message.into(),
        });
    }

    /// Push a warning diagnostic.
    pub fn push_warning(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.diagnostics.push(PipelineDiagnostic {
            level: DiagnosticLevel::Warning,
            code: code.into(),
            message: message.into(),
        });
    }

    /// Push an error diagnostic.
    pub fn push_error(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.diagnostics.push(PipelineDiagnostic {
            level: DiagnosticLevel::Error,
            code: code.into(),
            message: message.into(),
        });
    }

    /// Return true if any error diagnostics exist.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| matches!(d.level, DiagnosticLevel::Error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_params_and_diagnostics() {
        let mut ctx = PipelineContext::default();
        ctx.set_param("a", "b");
        assert_eq!(ctx.get_param("a"), Some("b"));

        ctx.push_info("i", "info");
        ctx.push_warning("w", "warn");
        ctx.push_error("e", "err");

        assert_eq!(ctx.diagnostics.len(), 3);
        assert!(ctx.has_errors());
    }
}
