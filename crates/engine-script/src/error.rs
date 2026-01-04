use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("script compilation failed: {message}")]
    CompilationError { message: String },

    #[error("script execution failed: {message}")]
    ExecutionError { message: String },

    #[error("invalid goto target: {target}")]
    InvalidGotoTarget { target: String },

    #[error("script exceeded operation limit")]
    OperationLimitExceeded,

    #[error("invalid argument to {function}: {message}")]
    InvalidArgument { function: String, message: String },
}

impl From<rhai::ParseError> for ScriptError {
    fn from(err: rhai::ParseError) -> Self {
        ScriptError::CompilationError {
            message: err.to_string(),
        }
    }
}

impl From<Box<rhai::EvalAltResult>> for ScriptError {
    fn from(err: Box<rhai::EvalAltResult>) -> Self {
        ScriptError::ExecutionError {
            message: err.to_string(),
        }
    }
}
