//! Built-in plugins for SIGNIA.
//!
//! Built-in plugins are included with the `builtin` feature.
//! They provide reference implementations and a practical baseline.
//!
//! Plugins in this module are deterministic and side-effect free by design.
//! Any I/O (reading files, network access) must be performed by the host layer,
//! and the plugin receives only structured inputs.

#![cfg(feature = "builtin")]

pub mod dataset;
pub mod openapi;
pub mod repo;
pub mod workflow;

use crate::registry::PluginRegistry;

/// Register all built-in plugins into the provided registry.
///
/// This function is deterministic: plugin ids are stable, and the registry uses
/// a `BTreeMap` internally.
pub fn register_all(registry: &mut PluginRegistry) {
    // Register in a stable order (even though registry is ordered).
    // This keeps logs and debugging consistent.
    repo::register(registry);
    openapi::register(registry);
    dataset::register(registry);
    workflow::register(registry);
}
