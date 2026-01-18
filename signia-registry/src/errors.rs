use anchor_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    #[msg("unauthorized")]
    Unauthorized,
    #[msg("invalid namespace")]
    InvalidNamespace,
    #[msg("invalid schema hash")]
    InvalidSchemaHash,
    #[msg("invalid kind")]
    InvalidKind,
    #[msg("invalid uri")]
    InvalidUri,
    #[msg("invalid version tag")]
    InvalidVersionTag,
    #[msg("entry revoked")]
    EntryRevoked,
}
