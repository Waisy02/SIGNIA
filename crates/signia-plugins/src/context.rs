//! Plugin execution context for SIGNIA.
//!
//! This module defines a stable context object passed to plugins.
//!
//! Goals:
//! - provide access to SIGNIA pipeline context and configuration
//! - enforce host capabilities and limits
//! - collect diagnostics deterministically
//! - provide structured inputs and outputs
//!
//! Non-goals:
//! - direct network/filesystem access
//! - ambient authority
//!
//! The host layer (CLI/API) is responsible for creating and populating this context.

use std::collections::BTreeMap;

use signia_core::pipeline::context::{PipelineContext, PipelineDiagnostic};

use crate::plugin::HostCapabilities;

/// Limits applied to plugin execution.
#[derive(Debug, Clone)]
pub struct PluginLimits {
    pub max_bytes: u64,
    pub max_nodes: u64,
    pub max_edges: u64,
    pub max_seconds: u64, // host-enforced; not measured here
}

impl Default for PluginLimits {
    fn default() -> Self {
        Self {
            max_bytes: 16 * 1024 * 1024,
            max_nodes: 100_000,
            max_edges: 200_000,
            max_seconds: 10,
        }
    }
}

/// Deterministic policy flags.
#[derive(Debug, Clone)]
pub struct PluginPolicy {
    /// Whether network is allowed.
    pub network: bool,
    /// Whether filesystem reads are allowed.
    pub filesystem: bool,
    /// Whether clock access is allowed.
    pub clock: bool,
    /// Whether child process spawn is allowed.
    pub spawn: bool,
}

impl Default for PluginPolicy {
    fn default() -> Self {
        Self {
            network: false,
            filesystem: false,
            clock: false,
            spawn: false,
        }
    }
}

impl PluginPolicy {
    pub fn from_host_caps(caps: &HostCapabilities) -> Self {
        Self {
            network: caps.network,
            filesystem: caps.filesystem,
            clock: caps.clock,
            spawn: caps.spawn,
        }
    }
}

/// Context passed into plugin execution.
///
/// This wraps the `signia-core` pipeline context and adds plugin-specific helpers.
pub struct PluginContext<'a> {
    /// Underlying pipeline context from signia-core.
    pub pipeline: &'a mut PipelineContext,

    /// Host capabilities.
    pub host_caps: HostCapabilities,

    /// Policy flags (host decisions).
    pub policy: PluginPolicy,

    /// Limits enforced by the host.
    pub limits: PluginLimits,

    /// Plugin-scoped key-value settings.
    pub settings: BTreeMap<String, String>,

    /// Diagnostics collected during plugin execution.
    pub diagnostics: Vec<PipelineDiagnostic>,
}

impl<'a> PluginContext<'a> {
    pub fn new(pipeline: &'a mut PipelineContext, host_caps: HostCapabilities) -> Self {
        Self {
            pipeline,
            host_caps,
            policy: PluginPolicy::from_host_caps(&host_caps),
            limits: PluginLimits::default(),
            settings: BTreeMap::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn with_limits(mut self, limits: PluginLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_setting(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.settings.insert(key.into(), value.into());
        self
    }

    pub fn emit_diag(&mut self, d: PipelineDiagnostic) {
        self.diagnostics.push(d);
    }

    pub fn take_diags(&mut self) -> Vec<PipelineDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use signia_core::pipeline::context::{DiagnosticLevel, PipelineConfig};

    #[test]
    fn plugin_context_constructs() {
        let cfg = PipelineConfig::default();
        let mut pipeline = PipelineContext::new(cfg);

        let mut ctx = PluginContext::new(
            &mut pipeline,
            HostCapabilities {
                network: false,
                filesystem: false,
                clock: false,
                spawn: false,
            },
        )
        .with_setting("x", "y");

        ctx.emit_diag(PipelineDiagnostic {
            level: DiagnosticLevel::Info,
            code: "note".to_string(),
            message: "hello".to_string(),
        });

        assert_eq!(ctx.settings.get("x").unwrap(), "y");
        assert_eq!(ctx.take_diags().len(), 1);
    }
}
