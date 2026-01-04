use holdsmith_parser::Span;
use thiserror::Error;

pub type CompileResult<T> = Result<T, CompileError>;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("unknown passage '{name}' referenced at {span:?}")]
    UnknownPassage { name: String, span: Span },

    #[error("duplicate passage name '{name}' at {span:?}")]
    DuplicatePassage { name: String, span: Span },

    #[error("choice has no target (missing -> or -> END) at {span:?}")]
    MissingChoiceTarget { span: Span },

    #[error("scene has no passages")]
    EmptyScene { span: Span },

    #[error("multiple errors occurred")]
    Multiple(Vec<CompileError>),
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("unknown resource '{resource}' in scene '{scene_id}'")]
    UnknownResource { resource: String, scene_id: String },

    #[error("unknown tag '{tag}' in category '{category}' in scene '{scene_id}'")]
    UnknownTag {
        tag: String,
        category: String,
        scene_id: String,
    },

    #[error("unknown context '{context}' in scene '{scene_id}'")]
    UnknownContext { context: String, scene_id: String },

    #[error("passage '{passage}' in scene '{scene_id}' has no choices and is not terminal")]
    DeadEndPassage { passage: String, scene_id: String },

    #[error("unreachable passage '{passage}' in scene '{scene_id}'")]
    UnreachablePassage { passage: String, scene_id: String },

    #[error("scene '{scene_id}' references unknown card '{card_id}'")]
    UnknownCard { card_id: String, scene_id: String },
}

#[derive(Debug, Default)]
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug)]
pub enum ValidationWarning {
    EmptyPassage {
        passage: String,
        scene_id: String,
    },
    HighDamageValue {
        resource: String,
        amount: i64,
        scene_id: String,
    },
    LowWeight {
        scene_id: String,
        weight: u32,
    },
}

impl ValidationReport {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn error(&mut self, err: ValidationError) {
        self.errors.push(err);
    }

    pub fn warn(&mut self, warn: ValidationWarning) {
        self.warnings.push(warn);
    }

    pub fn merge(&mut self, other: ValidationReport) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

impl CompileError {
    pub fn span(&self) -> Option<&Span> {
        match self {
            CompileError::UnknownPassage { span, .. } => Some(span),
            CompileError::DuplicatePassage { span, .. } => Some(span),
            CompileError::MissingChoiceTarget { span } => Some(span),
            CompileError::EmptyScene { span } => Some(span),
            CompileError::Multiple(_) => None,
        }
    }
}
