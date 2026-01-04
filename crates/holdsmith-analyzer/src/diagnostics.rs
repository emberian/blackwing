//! Diagnostic types for scene analysis results.
//!
//! Provides structured diagnostics that can be displayed to users,
//! written to files, or processed by tooling.

use smol_str::SmolStr;

use crate::analyzer::AnalysisResult;

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational message
    Info,
    /// Warning - potential issue
    Warning,
    /// Error - definite problem
    Error,
}

/// A diagnostic message from scene analysis.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: DiagnosticCode,
    pub message: String,
    pub scene_id: SmolStr,
    pub location: Option<DiagnosticLocation>,
    pub help: Option<String>,
}

/// Diagnostic codes for categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    /// Unreachable passage detected
    UnreachablePassage,
    /// Choice condition is always false
    ImpossibleChoice,
    /// Choice condition is always true (redundant)
    TautologicalChoice,
    /// Scene uses RNG (non-deterministic)
    UsesRng,
    /// Script compilation error
    ScriptError,
    /// Resource read without guaranteed initialization
    UninitializedRead,
    /// Scene has no exit paths
    NoExitPaths,
    /// Potential infinite loop
    PotentialLoop,
}

impl DiagnosticCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnreachablePassage => "HS001",
            Self::ImpossibleChoice => "HS002",
            Self::TautologicalChoice => "HS003",
            Self::UsesRng => "HS004",
            Self::ScriptError => "HS005",
            Self::UninitializedRead => "HS006",
            Self::NoExitPaths => "HS007",
            Self::PotentialLoop => "HS008",
        }
    }
}

/// Location within a scene where a diagnostic applies.
#[derive(Debug, Clone)]
pub enum DiagnosticLocation {
    Scene,
    SceneRequirements,
    Passage { index: usize, name: Option<SmolStr> },
    Choice { passage_index: usize, choice_index: usize, text: SmolStr },
}

impl std::fmt::Display for DiagnosticLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scene => write!(f, "scene"),
            Self::SceneRequirements => write!(f, "scene requirements"),
            Self::Passage { index, name } => {
                if let Some(n) = name {
                    write!(f, "passage {} ({})", index, n)
                } else {
                    write!(f, "passage {}", index)
                }
            }
            Self::Choice { passage_index, choice_index, text } => {
                write!(f, "choice {} in passage {} ({:?})", choice_index, passage_index, text)
            }
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let severity = match self.severity {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        };

        write!(f, "{} [{}]: {}", severity, self.code.as_str(), self.message)?;

        if let Some(ref loc) = self.location {
            write!(f, " (at {})", loc)?;
        }

        if let Some(ref help) = self.help {
            write!(f, "\n  help: {}", help)?;
        }

        Ok(())
    }
}

/// Convert analysis results into diagnostics.
pub fn generate_diagnostics(result: &AnalysisResult) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Unreachable passages
    for &passage_idx in &result.unreachable_passages {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: DiagnosticCode::UnreachablePassage,
            message: format!("Passage {} is unreachable from the entry point", passage_idx),
            scene_id: result.scene_id.clone(),
            location: Some(DiagnosticLocation::Passage {
                index: passage_idx,
                name: None,
            }),
            help: Some("Check if there should be a choice leading to this passage".to_string()),
        });
    }

    // Impossible choices
    for impossible in &result.impossible_choices {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: DiagnosticCode::ImpossibleChoice,
            message: format!(
                "Choice {:?} can never be taken: {}",
                impossible.choice_text, impossible.reason
            ),
            scene_id: result.scene_id.clone(),
            location: Some(DiagnosticLocation::Choice {
                passage_index: impossible.passage_index,
                choice_index: impossible.choice_index,
                text: impossible.choice_text.clone(),
            }),
            help: Some("Review the condition or remove this choice".to_string()),
        });
    }

    // Tautological choices
    for taut in &result.tautological_choices {
        diagnostics.push(Diagnostic {
            severity: Severity::Info,
            code: DiagnosticCode::TautologicalChoice,
            message: format!("Choice {:?} has a condition that is always true", taut.choice_text),
            scene_id: result.scene_id.clone(),
            location: Some(DiagnosticLocation::Choice {
                passage_index: taut.passage_index,
                choice_index: taut.choice_index,
                text: taut.choice_text.clone(),
            }),
            help: Some("Consider removing the redundant condition".to_string()),
        });
    }

    // RNG usage
    if result.uses_rng {
        diagnostics.push(Diagnostic {
            severity: Severity::Info,
            code: DiagnosticCode::UsesRng,
            message: "Scene uses random number generation".to_string(),
            scene_id: result.scene_id.clone(),
            location: Some(DiagnosticLocation::Scene),
            help: Some("This scene's outcomes cannot be fully analyzed statically".to_string()),
        });
    }

    // Script errors
    for error in &result.errors {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: DiagnosticCode::ScriptError,
            message: error.clone(),
            scene_id: result.scene_id.clone(),
            location: None,
            help: None,
        });
    }

    // Sort by severity (errors first)
    diagnostics.sort_by_key(|d| std::cmp::Reverse(d.severity));

    diagnostics
}

/// Summary of diagnostics by severity.
#[derive(Debug, Default)]
pub struct DiagnosticSummary {
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
}

impl DiagnosticSummary {
    pub fn from_diagnostics(diagnostics: &[Diagnostic]) -> Self {
        let mut summary = Self::default();
        for d in diagnostics {
            match d.severity {
                Severity::Error => summary.errors += 1,
                Severity::Warning => summary.warnings += 1,
                Severity::Info => summary.infos += 1,
            }
        }
        summary
    }

    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }

    pub fn is_clean(&self) -> bool {
        self.errors == 0 && self.warnings == 0
    }
}

impl std::fmt::Display for DiagnosticSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} error(s), {} warning(s), {} info(s)",
            self.errors, self.warnings, self.infos
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_diagnostic_display() {
        let diag = Diagnostic {
            severity: Severity::Warning,
            code: DiagnosticCode::UnreachablePassage,
            message: "Passage 3 is unreachable".to_string(),
            scene_id: "test_scene".into(),
            location: Some(DiagnosticLocation::Passage {
                index: 3,
                name: Some("hidden".into()),
            }),
            help: Some("Add a choice leading here".to_string()),
        };

        let s = diag.to_string();
        assert!(s.contains("warning"));
        assert!(s.contains("HS001"));
        assert!(s.contains("unreachable"));
    }

    #[test]
    fn test_generate_diagnostics() {
        let result = AnalysisResult {
            scene_id: "test".into(),
            unreachable_passages: vec![2, 3],
            uses_rng: true,
            reachable_passages: HashSet::from([0, 1]),
            ..Default::default()
        };

        let diagnostics = generate_diagnostics(&result);

        assert!(diagnostics.iter().any(|d| d.code == DiagnosticCode::UnreachablePassage));
        assert!(diagnostics.iter().any(|d| d.code == DiagnosticCode::UsesRng));
    }

    #[test]
    fn test_diagnostic_summary() {
        let diagnostics = vec![
            Diagnostic {
                severity: Severity::Error,
                code: DiagnosticCode::ScriptError,
                message: "error".to_string(),
                scene_id: "test".into(),
                location: None,
                help: None,
            },
            Diagnostic {
                severity: Severity::Warning,
                code: DiagnosticCode::UnreachablePassage,
                message: "warning".to_string(),
                scene_id: "test".into(),
                location: None,
                help: None,
            },
            Diagnostic {
                severity: Severity::Warning,
                code: DiagnosticCode::ImpossibleChoice,
                message: "warning2".to_string(),
                scene_id: "test".into(),
                location: None,
                help: None,
            },
        ];

        let summary = DiagnosticSummary::from_diagnostics(&diagnostics);
        assert_eq!(summary.errors, 1);
        assert_eq!(summary.warnings, 2);
        assert!(summary.has_errors());
        assert!(!summary.is_clean());
    }
}
