//! Path normalization utilities for SIGNIA.
//!
//! This module defines deterministic path normalization rules used when
//! compiling structures that originate from filesystems, archives, or
//! virtual path sources.
//!
//! Goals:
//! - identical logical paths produce identical normalized representations
//! - no dependency on OS-specific path semantics
//! - no filesystem access
//!
//! Non-goals:
//! - resolving symlinks on disk
//! - checking path existence
//!
//! All normalization is purely string-based.

use crate::errors::{SigniaError, SigniaResult};

/// Normalize a logical path into a canonical form.
///
/// Rules:
/// - backslashes are converted to forward slashes
/// - repeated slashes are collapsed
/// - "." segments are removed
/// - ".." segments are resolved without escaping root
/// - leading slash is preserved if present
/// - trailing slash is removed unless path is root
///
/// This function does not perform percent-decoding or encoding.
pub fn normalize_path(input: &str) -> SigniaResult<String> {
    if input.is_empty() {
        return Err(SigniaError::invalid_argument("path is empty"));
    }

    let mut s = input.replace('\\', "/");

    // Collapse repeated slashes
    while s.contains("//") {
        s = s.replace("//", "/");
    }

    let is_absolute = s.starts_with('/');

    let mut parts = Vec::new();
    for part in s.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if let Some(last) = parts.pop() {
                    let _ = last;
                }
            }
            p => parts.push(p),
        }
    }

    let mut out = String::new();
    if is_absolute {
        out.push('/');
    }
    out.push_str(&parts.join("/"));

    if out.len() > 1 && out.ends_with('/') {
        out.pop();
    }

    Ok(out)
}

/// Normalize a path under a declared root.
///
/// Ensures the normalized path does not escape the root.
pub fn normalize_under_root(root: &str, path: &str) -> SigniaResult<String> {
    let root_n = normalize_path(root)?;
    let path_n = normalize_path(path)?;

    if !path_n.starts_with(&root_n) {
        return Err(SigniaError::invalid_argument(
            "path escapes declared root",
        ));
    }

    Ok(path_n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_basic() {
        assert_eq!(normalize_path("a/b/c").unwrap(), "a/b/c");
        assert_eq!(normalize_path("a//b///c").unwrap(), "a/b/c");
        assert_eq!(normalize_path("a/./b/../c").unwrap(), "a/c");
    }

    #[test]
    fn normalize_absolute() {
        assert_eq!(normalize_path("/a/b/").unwrap(), "/a/b");
        assert_eq!(normalize_path("/a/../b").unwrap(), "/b");
    }

    #[test]
    fn normalize_under_root_ok() {
        let r = normalize_under_root("/root", "/root/a/b").unwrap();
        assert_eq!(r, "/root/a/b");
    }

    #[test]
    fn normalize_under_root_err() {
        let err = normalize_under_root("/root", "/other/x").err().unwrap();
        assert!(err.to_string().contains("escapes"));
    }
}
