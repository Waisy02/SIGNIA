//! signia-plugins
//!
//! This crate provides the plugin system for SIGNIA, including:
//! - plugin trait and execution interface
//! - plugin registry and resolution
//! - built-in plugins (feature: `builtin`)
//! - optional WASM sandbox runner (feature: `wasm`)
//!
//! Design principles:
//! - deterministic execution: same input -> same output
//! - explicit I/O boundaries: plugins receive structured inputs only
//! - no hidden network/filesystem/time access (unless explicitly allowed by host)
//!
//! The plugin system is intentionally minimal to keep the trusted surface small.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod plugin;
pub mod registry;

#[cfg(feature = "builtin")]
pub mod builtin;

#[cfg(feature = "wasm")]
pub mod sandbox;

pub use plugin::{
    HostCapabilities, Plugin, PluginError, PluginInput, PluginOutput, PluginResult,
    PluginVersion,
};
pub use registry::{PluginRegistry, PluginResolver, ResolvedPlugin};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Convenience: create a default registry with built-in plugins registered.
///
/// This is typically used by CLI and API layers.
#[cfg(feature = "builtin")]
pub fn default_registry() -> PluginRegistry {
    let mut reg = PluginRegistry::new();
    builtin::register_all(&mut reg);
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_nonempty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    #[cfg(feature = "builtin")]
    fn default_registry_has_plugins() {
        let reg = default_registry();
        assert!(reg.len() > 0);
    }
}
