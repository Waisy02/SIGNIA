//! Diagnostics for SIGNIA.
//!
//! Diagnostics are structured messages emitted during parsing, inference,
//! compilation, and verification.
//!
//! Principles:
//! - deterministic: no implicit timestamps, no machine-specific data
//! - structured: codes + fields for tooling and filtering
//! - composable: can be attached to IR nodes/edges, pipeline reports, API responses
//!
//! This module defines:
//! - diagnostic levels
//! - diagnostic codes
//! - helpers for building and attaching diagnostics
//! - conversion helpers to pipeline diagnostics (for consistent UX)

use std::collections::BTreeMap;

use crate::errors::{SigniaError, SigniaResult};

/// Severity level for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagLevel {
    Info,
    Warning,
    Error,
}

impl DiagLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagLevel::Info => "info",
            DiagLevel::Warning => "warning",
            DiagLevel::Error => "error",
        }
    }
}

/// A structured diagnostic message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: DiagLevel,
    pub code: String,
    pub message: String,
    pub fields: BTreeMap<String, String>,
}

impl Diagnostic {
    pub fn new(level: DiagLevel, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level,
            code: code.into(),
            message: message.into(),
            fields: BTreeMap::new(),
        }
    }

    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    pub fn is_error(&self) -> bool {
        matches!(self.level, DiagLevel::Error)
    }

    pub fn is_warning(&self) -> bool {
        matches!(self.level, DiagLevel::Warning)
    }
}

/// A diagnostics collection.
#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    pub items: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn push(&mut self, d: Diagnostic) {
        self.items.push(d);
    }

    pub fn extend(&mut self, other: Diagnostics) {
        self.items.extend(other.items);
    }

    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|d| d.is_error())
    }

    pub fn has_warnings(&self) -> bool {
        self.items.iter().any(|d| d.is_warning())
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }
}

/// Helper constructors for common diagnostics.
pub mod codes {
    use super::*;

    pub fn invalid_argument(msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(DiagLevel::Error, "invalid_argument", msg)
    }

    pub fn invalid_schema(msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(DiagLevel::Error, "invalid_schema", msg)
    }

    pub fn invalid_manifest(msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(DiagLevel::Error, "invalid_manifest", msg)
    }

    pub fn determinism_violation(msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(DiagLevel::Error, "determinism_violation", msg)
    }

    pub fn limit_exceeded(msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(DiagLevel::Error, "limit_exceeded", msg)
    }

    pub fn unsupported(msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(DiagLevel::Error, "unsupported", msg)
    }

    pub fn note(msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(DiagLevel::Info, "note", msg)
    }

    pub fn warn(msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(DiagLevel::Warning, "warning", msg)
    }
}

/// Convert SIGNIA diagnostics to pipeline diagnostics.
pub fn to_pipeline_diagnostic(d: &Diagnostic) -> crate::pipeline::context::PipelineDiagnostic {
    use crate::pipeline::context::{DiagnosticLevel, PipelineDiagnostic};

    let level = match d.level {
        DiagLevel::Info => DiagnosticLevel::Info,
        DiagLevel::Warning => DiagnosticLevel::Warning,
        DiagLevel::Error => DiagnosticLevel::Error,
    };

    PipelineDiagnostic {
        level,
        code: d.code.clone(),
        message: d.message.clone(),
    }
}

/// Convert pipeline diagnostics to SIGNIA diagnostics.
pub fn from_pipeline_diagnostic(
    d: &crate::pipeline::context::PipelineDiagnostic,
) -> Diagnostic {
    let level = match d.level {
        crate::pipeline::context::DiagnosticLevel::Info => DiagLevel::Info,
        crate::pipeline::context::DiagnosticLevel::Warning => DiagLevel::Warning,
        crate::pipeline::context::DiagnosticLevel::Error => DiagLevel::Error,
    };

    Diagnostic::new(level, d.code.clone(), d.message.clone())
}

/// Utility: fail if diagnostics has errors.
pub fn fail_if_errors(diags: &Diagnostics) -> SigniaResult<()> {
    if diags.has_errors() {
        return Err(SigniaError::invariant("diagnostics contains errors"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_basic() {
        let mut d = Diagnostics::default();
        d.push(Diagnostic::new(DiagLevel::Info, "x", "hello"));
        d.push(Diagnostic::new(DiagLevel::Warning, "y", "warn"));
        assert!(d.has_warnings());
        assert!(!d.has_errors());
    }

    #[test]
    fn conversion_roundtrip() {
        let d = Diagnostic::new(DiagLevel::Error, "code", "msg");
        let p = to_pipeline_diagnostic(&d);
        let back = from_pipeline_diagnostic(&p);
        assert_eq!(back.level, DiagLevel::Error);
        assert_eq!(back.code, "code");
        assert_eq!(back.message, "msg");
    }
}
