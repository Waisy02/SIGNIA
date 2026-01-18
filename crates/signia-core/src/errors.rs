//! Error types for signia-core.
//!
//! Errors are structured, explicit, and stable. Messages are intended to be
//! human-readable while preserving machine-level categorization.

use std::fmt::{self, Display};

/// Result type used throughout signia-core.
pub type SigniaResult<T> = Result<T, SigniaError>;

/// Top-level error type for signia-core.
#[derive(Debug)]
pub enum SigniaError {
    /// Invalid or unsupported argument.
    InvalidArgument {
        message: String,
    },

    /// Canonicalization failure.
    Canonicalization {
        message: String,
    },

    /// Hashing failure.
    Hashing {
        message: String,
    },

    /// Merkle tree construction or verification failure.
    Merkle {
        message: String,
    },

    /// Path normalization or validation failure.
    Path {
        message: String,
    },

    /// Serialization or deserialization failure.
    Serialization {
        message: String,
    },

    /// Internal invariant violation.
    Invariant {
        message: String,
    },
}

impl SigniaError {
    /// Construct an invalid argument error.
    pub fn invalid_argument<M: Into<String>>(message: M) -> Self {
        Self::InvalidArgument {
            message: message.into(),
        }
    }

    /// Construct a canonicalization error.
    pub fn canonicalization<M: Into<String>>(message: M) -> Self {
        Self::Canonicalization {
            message: message.into(),
        }
    }

    /// Construct a hashing error.
    pub fn hashing<M: Into<String>>(message: M) -> Self {
        Self::Hashing {
            message: message.into(),
        }
    }

    /// Construct a merkle error.
    pub fn merkle<M: Into<String>>(message: M) -> Self {
        Self::Merkle {
            message: message.into(),
        }
    }

    /// Construct a path error.
    pub fn path<M: Into<String>>(message: M) -> Self {
        Self::Path {
            message: message.into(),
        }
    }

    /// Construct a serialization error.
    pub fn serialization<M: Into<String>>(message: M) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }

    /// Construct an invariant violation error.
    pub fn invariant<M: Into<String>>(message: M) -> Self {
        Self::Invariant {
            message: message.into(),
        }
    }
}

impl Display for SigniaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument { message } => {
                write!(f, "invalid argument: {message}")
            }
            Self::Canonicalization { message } => {
                write!(f, "canonicalization error: {message}")
            }
            Self::Hashing { message } => {
                write!(f, "hashing error: {message}")
            }
            Self::Merkle { message } => {
                write!(f, "merkle error: {message}")
            }
            Self::Path { message } => {
                write!(f, "path error: {message}")
            }
            Self::Serialization { message } => {
                write!(f, "serialization error: {message}")
            }
            Self::Invariant { message } => {
                write!(f, "invariant violation: {message}")
            }
        }
    }
}

impl std::error::Error for SigniaError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_invalid_argument() {
        let e = SigniaError::invalid_argument("bad input");
        assert_eq!(format!("{e}"), "invalid argument: bad input");
    }

    #[test]
    fn display_hashing_error() {
        let e = SigniaError::hashing("digest mismatch");
        assert_eq!(format!("{e}"), "hashing error: digest mismatch");
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SigniaError>();
    }
}
