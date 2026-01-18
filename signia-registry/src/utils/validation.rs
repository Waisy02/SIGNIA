use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::RegistryError;

pub fn validate_namespace(ns: &str) -> Result<()> {
    let s = ns.trim();
    if s.is_empty() || s.len() > MAX_NAMESPACE_LEN {
        return Err(error!(RegistryError::InvalidNamespace));
    }
    // Require already-normalized namespace to make PDA derivation stable across clients.
    // Allowed charset: [a-z0-9-]
    if !s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(error!(RegistryError::InvalidNamespace));
    }
    Ok(())
}

pub fn validate_kind(kind: &str) -> Result<()> {
    if kind.len() > MAX_KIND_LEN {
        return Err(error!(RegistryError::InvalidKind));
    }
    Ok(())
}

pub fn validate_uri(uri: &str) -> Result<()> {
    if uri.len() > MAX_URI_LEN {
        return Err(error!(RegistryError::InvalidUri));
    }
    Ok(())
}

pub fn validate_version_tag(v: &str) -> Result<()> {
    if v.len() > MAX_VERSION_TAG_LEN {
        return Err(error!(RegistryError::InvalidVersionTag));
    }
    Ok(())
}
