//! Error types for game bundles.

use thiserror::Error;

/// Errors that can occur when working with game bundles.
#[derive(Debug, Error)]
pub enum BundleError {
    /// I/O error (reading/writing files)
    #[error("I/O error: {0}")]
    Io(String),

    /// Parse error (TOML, YAML, etc.)
    #[error("Parse error: {0}")]
    Parse(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialize(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Content not found
    #[error("Content not found: {0}")]
    NotFound(String),
}
