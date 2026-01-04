use thiserror::Error;

use crate::Span;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("invalid frontmatter YAML: {message}")]
    InvalidFrontmatter { message: String, span: Span },

    #[error("invalid condition: {message}")]
    InvalidCondition { message: String, span: Span },

    #[error("invalid effect: {message}")]
    InvalidEffect { message: String, span: Span },

    #[error("unexpected end of file")]
    UnexpectedEof { span: Span },
}

impl ParseError {
    pub fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken { span, .. } => span.clone(),
            ParseError::InvalidFrontmatter { span, .. } => span.clone(),
            ParseError::InvalidCondition { span, .. } => span.clone(),
            ParseError::InvalidEffect { span, .. } => span.clone(),
            ParseError::UnexpectedEof { span } => span.clone(),
        }
    }
}
