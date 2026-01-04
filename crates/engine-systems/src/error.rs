//! Error types for the systems crate.

use smol_str::SmolStr;
use thiserror::Error;

/// Errors that can occur during system operations.
#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Unknown command kind: {0}")]
    UnknownCommand(SmolStr),

    #[error("Missing required argument: {0}")]
    MissingArgument(SmolStr),

    #[error("Invalid argument type for {key}: expected {expected}, got {actual}")]
    InvalidArgumentType {
        key: SmolStr,
        expected: &'static str,
        actual: &'static str,
    },

    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("System not active in current scope")]
    NotInScope,

    #[error("Script error: {0}")]
    ScriptError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl SystemError {
    pub fn missing_arg(key: impl Into<SmolStr>) -> Self {
        Self::MissingArgument(key.into())
    }

    pub fn entity_not_found(id: impl std::fmt::Display) -> Self {
        Self::EntityNotFound(id.to_string())
    }

    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Self::InvalidState(msg.into())
    }
}
