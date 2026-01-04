//! View models for UI rendering.

use holdsmith_analyzer::{Diagnostic, DiagnosticLocation, Severity};

/// A node in the file tree.
#[derive(Debug, Clone)]
pub struct FileTreeNode {
    /// File or directory name.
    pub name: String,
    /// Full path from root.
    pub path: String,
    /// Whether this is a directory.
    pub is_dir: bool,
    /// Children (if directory).
    pub children: Vec<FileTreeNode>,
    /// Whether expanded in UI.
    pub expanded: bool,
}

/// View model for a diagnostic.
#[derive(Debug, Clone)]
pub struct DiagnosticViewModel {
    pub severity: SeverityViewModel,
    pub code: String,
    pub message: String,
    pub location: Option<LocationViewModel>,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum SeverityViewModel {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct LocationViewModel {
    pub description: String,
    pub passage_index: Option<usize>,
    pub choice_index: Option<usize>,
}

impl From<&Diagnostic> for DiagnosticViewModel {
    fn from(d: &Diagnostic) -> Self {
        Self {
            severity: match d.severity {
                Severity::Error => SeverityViewModel::Error,
                Severity::Warning => SeverityViewModel::Warning,
                Severity::Info => SeverityViewModel::Info,
            },
            code: format!("{:?}", d.code),
            message: d.message.clone(),
            location: d.location.as_ref().map(|loc| match loc {
                DiagnosticLocation::Scene => LocationViewModel {
                    description: "Scene".to_string(),
                    passage_index: None,
                    choice_index: None,
                },
                DiagnosticLocation::SceneRequirements => LocationViewModel {
                    description: "Scene requirements".to_string(),
                    passage_index: None,
                    choice_index: None,
                },
                DiagnosticLocation::Passage { index, name } => LocationViewModel {
                    description: name
                        .as_ref()
                        .map(|n| format!("Passage '{}' ({})", n, index))
                        .unwrap_or_else(|| format!("Passage {}", index)),
                    passage_index: Some(*index),
                    choice_index: None,
                },
                DiagnosticLocation::Choice {
                    passage_index,
                    choice_index,
                    text,
                } => LocationViewModel {
                    description: format!("Choice '{}' in passage {}", text, passage_index),
                    passage_index: Some(*passage_index),
                    choice_index: Some(*choice_index),
                },
            }),
            help: d.help.clone(),
        }
    }
}

/// View model for a passage (for CFG visualization).
#[derive(Debug, Clone)]
pub struct PassageViewModel {
    pub index: usize,
    pub name: String,
    pub is_entry: bool,
    pub is_exit: bool,
    pub is_reachable: bool,
    pub has_errors: bool,
    pub has_warnings: bool,
    pub choice_count: usize,
}

/// View model for a choice (edge in CFG).
#[derive(Debug, Clone)]
pub struct ChoiceViewModel {
    pub index: usize,
    pub text: String,
    pub from_passage: usize,
    pub to_passage: Option<usize>, // None if -> END
    pub has_condition: bool,
    pub is_impossible: bool,
    pub is_tautological: bool,
}

/// State inspector view model.
#[derive(Debug, Clone, Default)]
pub struct StateInspector {
    pub resources: Vec<ResourceEntry>,
    pub flags: Vec<FlagEntry>,
}

#[derive(Debug, Clone)]
pub struct ResourceEntry {
    pub name: String,
    pub value: i64,
    pub max: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct FlagEntry {
    pub name: String,
    pub value: String,
}
