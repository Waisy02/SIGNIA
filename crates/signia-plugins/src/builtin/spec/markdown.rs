//! Markdown rendering for plugin specs.
//!
//! This is used by docs generators and CLIs to print built-in plugin capabilities.
//!
//! Determinism rules:
//! - stable ordering for lists/maps
//! - stable formatting (no timestamps, no env-dependent data)

#![cfg(feature = "builtin")]

use std::collections::BTreeMap;

use crate::spec::PluginSpec;

/// Render a single `PluginSpec` as Markdown.
pub fn render_spec_markdown(spec: &PluginSpec) -> String {
    let mut out = String::new();

    out.push_str(&format!("# {}\n\n", escape_md(&spec.title)));
    out.push_str(&format!("**ID:** `{}`  \n", spec.id));
    out.push_str(&format!("**Version:** `{}`\n\n", spec.version));

    if !spec.description.is_empty() {
        out.push_str(&format!("{}\n\n", spec.description.trim()));
    }

    out.push_str("## Supports\n\n");
    if spec.supports.is_empty() {
        out.push_str("- _none_\n\n");
    } else {
        for s in &spec.supports {
            out.push_str(&format!("- `{}`\n", s));
        }
        out.push('\n');
    }

    out.push_str("## Limits\n\n");
    if spec.limits.is_empty() {
        out.push_str("- _none_\n\n");
    } else {
        // limits is BTreeMap => deterministic order
        for (k, v) in &spec.limits {
            out.push_str(&format!("- `{}`: `{}`\n", k, v));
        }
        out.push('\n');
    }

    out.push_str("## Capabilities\n\n");
    if spec.wants.is_empty() {
        out.push_str("- _none_\n\n");
    } else {
        for (k, v) in &spec.wants {
            out.push_str(&format!("- `{}`: `{}`\n", k, v));
        }
        out.push('\n');
    }

    out.push_str("## Metadata\n\n");
    if spec.meta.is_empty() {
        out.push_str("- _none_\n\n");
    } else {
        for (k, v) in &spec.meta {
            out.push_str(&format!("- `{}`: `{}`\n", k, escape_md(v)));
        }
        out.push('\n');
    }

    out.push_str("## Usage\n\n");
    out.push_str("This spec is intended for compatibility checks and documentation. ");
    out.push_str("Runtime behavior is implemented by the plugin code and enforced by the host.\n");

    out
}

/// Render multiple specs as a Markdown index page.
pub fn render_index_markdown(specs: &[PluginSpec]) -> String {
    let mut out = String::new();
    out.push_str("# Built-in Plugins\n\n");
    out.push_str("This page lists built-in SIGNIA plugin specifications.\n\n");

    for spec in specs {
        out.push_str(&format!("## {}\n\n", escape_md(&spec.title)));
        out.push_str(&format!("- **ID:** `{}`\n", spec.id));
        out.push_str(&format!("- **Version:** `{}`\n", spec.version));
        if !spec.supports.is_empty() {
            out.push_str("- **Supports:** ");
            out.push_str(
                &spec
                    .supports
                    .iter()
                    .map(|s| format!("`{}`", s))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            out.push('\n');
        }
        out.push('\n');
    }

    out
}

/// Render specs into a Markdown table (compact for README).
pub fn render_specs_table(specs: &[PluginSpec]) -> String {
    let mut out = String::new();
    out.push_str("| Plugin | ID | Version | Supports |\n");
    out.push_str("|---|---|---|---|\n");
    for spec in specs {
        let supports = if spec.supports.is_empty() {
            "_none_".to_string()
        } else {
            spec.supports.join(", ")
        };
        out.push_str(&format!(
            "| {} | `{}` | `{}` | `{}` |\n",
            escape_md(&spec.title),
            spec.id,
            spec.version,
            supports
        ));
    }
    out
}

/// Escape common Markdown special chars for stable output.
fn escape_md(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.' | '!' | '|' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Utility to render a BTreeMap as a Markdown list.
pub fn render_kv_list(map: &BTreeMap<String, String>) -> String {
    if map.is_empty() {
        return "- _none_\n".to_string();
    }
    let mut out = String::new();
    for (k, v) in map {
        out.push_str(&format!("- `{}`: `{}`\n", k, escape_md(v)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::PluginSpec;

    #[test]
    fn markdown_contains_id() {
        let spec = PluginSpec::new("x", "X", "0.1.0").support("repo");
        let md = render_spec_markdown(&spec);
        assert!(md.contains("**ID:** `x`"));
    }

    #[test]
    fn table_renders_rows() {
        let specs = vec![
            PluginSpec::new("a", "A", "0.1.0").support("repo"),
            PluginSpec::new("b", "B", "0.1.0").support("dataset"),
        ];
        let t = render_specs_table(&specs);
        assert!(t.contains("| A | `a` | `0.1.0` | `repo` |"));
    }
}
