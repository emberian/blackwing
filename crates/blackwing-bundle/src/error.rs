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

    /// Script compilation error
    #[error("Script error: {0}")]
    Script(String),

    /// Archive error (zip operations)
    #[error("Archive error: {0}")]
    Archive(String),
}

impl From<std::io::Error> for BundleError {
    fn from(e: std::io::Error) -> Self {
        BundleError::Io(e.to_string())
    }
}

impl From<toml::de::Error> for BundleError {
    fn from(e: toml::de::Error) -> Self {
        BundleError::Parse(e.to_string())
    }
}

impl From<toml::ser::Error> for BundleError {
    fn from(e: toml::ser::Error) -> Self {
        BundleError::Serialize(e.to_string())
    }
}

#[cfg(feature = "archive")]
impl From<zip::result::ZipError> for BundleError {
    fn from(e: zip::result::ZipError) -> Self {
        BundleError::Archive(e.to_string())
    }
}
