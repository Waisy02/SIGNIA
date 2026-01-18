//! Pipeline execution report for SIGNIA.
//!
//! A PipelineReport is returned after running a Pipeline.
//! It captures:
//! - final output data
//! - all diagnostics emitted by stages
//! - execution metadata (stage order, timing hooks)
//!
//! This struct is intentionally simple and serializable so it can be:
//! - printed by CLI
//! - returned by API
//! - persisted as an artifact
//!
//! Timing fields are placeholders for higher layers; core does not read clocks.

use crate::pipeline::context::PipelineDiagnostic;
use crate::pipeline::PipelineData;

/// Report produced after pipeline execution.
#[derive(Debug, Clone)]
pub struct PipelineReport {
    /// Final output of the pipeline.
    pub output: PipelineData,

    /// Diagnostics collected during execution.
    pub diagnostics: Vec<PipelineDiagnostic>,

    /// Ordered list of executed stage ids.
    pub stages: Vec<String>,
}

impl PipelineReport {
    pub fn new(output: PipelineData, diagnostics: Vec<PipelineDiagnostic>, stages: Vec<String>) -> Self {
        Self {
            output,
            diagnostics,
            stages,
        }
    }

    /// Returns true if any error diagnostics exist.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| matches!(d.level, crate::pipeline::context::DiagnosticLevel::Error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::context::{DiagnosticLevel, PipelineDiagnostic};

    #[test]
    fn report_basic() {
        let report = PipelineReport::new(
            PipelineData::None,
            vec![PipelineDiagnostic {
                level: DiagnosticLevel::Info,
                code: "test".to_string(),
                message: "ok".to_string(),
            }],
            vec!["stage1".to_string(), "stage2".to_string()],
        );

        assert_eq!(report.stages.len(), 2);
        assert!(!report.has_errors());
    }
}
