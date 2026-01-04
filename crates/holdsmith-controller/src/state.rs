//! Application state types.

use std::collections::{HashMap, HashSet};

use engine_core::Scene;
use holdsmith_analyzer::{AnalysisResult, Diagnostic, SceneCfg};
use holdsmith_parser::SceneFile;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use vfs::VfsPath;

use crate::full_player::{FullPlayerSnapshot, FullPlayerState};
use crate::view_models::{DiagnosticViewModel, FileTreeNode};

/// Complete application state.
#[derive(Debug)]
pub struct AppState {
    pub project: ProjectState,
    pub editor: EditorState,
    pub analyzer: AnalyzerState,
    pub debugger: DebuggerState,
    pub player: PlayerState,
    pub full_player: FullPlayerState,
}

impl AppState {
    pub fn new(fs: VfsPath) -> Self {
        let mut project = ProjectState::new(fs);
        project.refresh_file_tree();

        Self {
            project,
            editor: EditorState::default(),
            analyzer: AnalyzerState::default(),
            debugger: DebuggerState::default(),
            player: PlayerState::default(),
            full_player: FullPlayerState::default(),
        }
    }
}

/// Project state: files and filesystem.
#[derive(Debug)]
pub struct ProjectState {
    /// The virtual filesystem root.
    pub fs: VfsPath,
    /// Currently open files (paths).
    pub open_files: Vec<String>,
    /// The active file being edited.
    pub active_file: Option<String>,
    /// Files with unsaved changes.
    pub dirty_files: HashSet<String>,
    /// File tree for the browser panel.
    pub file_tree: Vec<FileTreeNode>,
}

impl ProjectState {
    pub fn new(fs: VfsPath) -> Self {
        Self {
            fs,
            open_files: Vec::new(),
            active_file: None,
            dirty_files: HashSet::new(),
            file_tree: Vec::new(),
        }
    }

    /// Refresh the file tree from the filesystem.
    pub fn refresh_file_tree(&mut self) {
        self.file_tree = build_file_tree(&self.fs);
    }

    /// Check if a file exists.
    pub fn file_exists(&self, path: &str) -> bool {
        self.fs
            .join(path)
            .map(|p| p.exists().unwrap_or(false))
            .unwrap_or(false)
    }

    /// Read a file's content.
    pub fn read_file(&self, path: &str) -> Option<String> {
        self.fs
            .join(path)
            .ok()
            .and_then(|p| p.read_to_string().ok())
    }

    /// Write content to a file.
    pub fn write_file(&mut self, path: &str, content: &str) -> Result<(), String> {
        let file_path = self.fs.join(path).map_err(|e| e.to_string())?;

        // Ensure parent directory exists
        let parent = file_path.parent();
        parent.create_dir_all().map_err(|e| e.to_string())?;

        file_path
            .create_file()
            .map_err(|e| e.to_string())?
            .write_all(content.as_bytes())
            .map_err(|e| e.to_string())?;

        self.refresh_file_tree();
        Ok(())
    }
}

/// Build a file tree from a VFS path.
fn build_file_tree(root: &VfsPath) -> Vec<FileTreeNode> {
    fn build_recursive(path: &VfsPath) -> Vec<FileTreeNode> {
        let mut nodes = Vec::new();

        if let Ok(entries) = path.read_dir() {
            let mut entries: Vec<_> = entries.collect();
            // Sort: directories first, then alphabetically
            entries.sort_by(|a, b| {
                let a_is_dir = a.is_dir().unwrap_or(false);
                let b_is_dir = b.is_dir().unwrap_or(false);
                match (a_is_dir, b_is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.filename().cmp(&b.filename()),
                }
            });

            for entry in entries {
                let name = entry.filename();
                let path_str = entry.as_str().to_string();
                let is_dir = entry.is_dir().unwrap_or(false);

                let children = if is_dir {
                    build_recursive(&entry)
                } else {
                    Vec::new()
                };

                nodes.push(FileTreeNode {
                    name,
                    path: path_str,
                    is_dir,
                    children,
                    expanded: false,
                });
            }
        }

        nodes
    }

    build_recursive(root)
}

/// Editor state: current file content and cursor.
#[derive(Debug, Default)]
pub struct EditorState {
    /// Current file content (if a file is open).
    pub content: String,
    /// Cursor position (line, column).
    pub cursor: (usize, usize),
    /// Selection range (start, end) in character offsets.
    pub selection: Option<(usize, usize)>,
    /// Parsed AST of current file (if valid).
    pub parsed_ast: Option<SceneFile>,
    /// Parse errors (if parsing failed).
    pub parse_errors: Vec<String>,
    /// Diagnostics from the analyzer.
    pub diagnostics: Vec<Diagnostic>,
}

/// Analyzer state: analysis results and CFG.
#[derive(Debug, Default)]
pub struct AnalyzerState {
    /// Analysis results per scene.
    pub results: HashMap<SmolStr, AnalysisResult>,
    /// Current scene's CFG (if analyzed).
    pub cfg: Option<SceneCfg>,
    /// Whether analysis is in progress.
    pub analyzing: bool,
    /// Compiled scenes (for player/debugger).
    pub compiled_scenes: HashMap<SmolStr, Scene>,
}

/// Debugger state: breakpoints and execution state.
#[derive(Debug, Default, Clone)]
pub struct DebuggerState {
    /// Whether the debugger is active.
    pub active: bool,
    /// Current scene being debugged.
    pub scene_id: Option<SmolStr>,
    /// Current passage index.
    pub passage_index: Option<usize>,
    /// Breakpoints (scene_id, passage_index).
    pub breakpoints: HashSet<(SmolStr, usize)>,
    /// Whether execution is paused.
    pub paused: bool,
    /// Passage history (for "call stack" view).
    pub passage_history: Vec<usize>,
}

/// Location in the debugger.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DebugLocation {
    pub scene_id: SmolStr,
    pub passage_index: usize,
}

/// Player state: scene playback for testing.
#[derive(Debug, Default, Clone)]
pub struct PlayerState {
    /// Whether the player is active.
    pub active: bool,
    /// Current scene being played.
    pub scene_id: Option<SmolStr>,
    /// Current passage index.
    pub passage_index: usize,
    /// Current passage text.
    pub passage_text: String,
    /// Available choices.
    pub choices: Vec<PlayerChoice>,
    /// Play history (for restart).
    pub history: Vec<PlayerHistoryEntry>,
}

/// A choice in the player.
#[derive(Debug, Clone)]
pub struct PlayerChoice {
    pub index: usize,
    pub text: String,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
}

/// A history entry in the player.
#[derive(Debug, Clone)]
pub struct PlayerHistoryEntry {
    pub passage_index: usize,
    pub choice_index: Option<usize>,
}

// ============================================================================
// Serializable Snapshots (for IPC)
// ============================================================================

/// Serializable snapshot of the entire application state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppStateSnapshot {
    pub project: ProjectSnapshot,
    pub editor: EditorSnapshot,
    pub analyzer: AnalyzerSnapshot,
    pub debugger: DebuggerSnapshot,
    pub player: PlayerSnapshot,
    pub full_player: FullPlayerSnapshot,
}

/// Snapshot of project state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub open_files: Vec<String>,
    pub active_file: Option<String>,
    pub dirty_files: Vec<String>,
    pub file_tree: Vec<FileTreeNode>,
}

/// Snapshot of editor state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorSnapshot {
    pub content: String,
    pub cursor: (usize, usize),
    pub selection: Option<(usize, usize)>,
    pub parse_errors: Vec<String>,
    pub diagnostics: Vec<DiagnosticViewModel>,
}

/// Snapshot of analyzer state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyzerSnapshot {
    pub analyzing: bool,
    pub compiled_scene_ids: Vec<String>,
    pub has_cfg: bool,
}

/// Snapshot of debugger state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DebuggerSnapshot {
    pub active: bool,
    pub scene_id: Option<String>,
    pub passage_index: Option<usize>,
    pub paused: bool,
    pub breakpoints: Vec<(String, usize)>,
}

/// Snapshot of player state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub active: bool,
    pub scene_id: Option<String>,
    pub passage_index: usize,
    pub passage_text: String,
    pub choices: Vec<PlayerChoiceSnapshot>,
}

/// Snapshot of a player choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerChoiceSnapshot {
    pub index: usize,
    pub text: String,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
}

impl AppState {
    /// Create a serializable snapshot of the current state.
    pub fn snapshot(&self) -> AppStateSnapshot {
        AppStateSnapshot {
            project: ProjectSnapshot {
                open_files: self.project.open_files.clone(),
                active_file: self.project.active_file.clone(),
                dirty_files: self.project.dirty_files.iter().cloned().collect(),
                file_tree: self.project.file_tree.clone(),
            },
            editor: EditorSnapshot {
                content: self.editor.content.clone(),
                cursor: self.editor.cursor,
                selection: self.editor.selection,
                parse_errors: self.editor.parse_errors.clone(),
                diagnostics: self.editor.diagnostics.iter().map(DiagnosticViewModel::from).collect(),
            },
            analyzer: AnalyzerSnapshot {
                analyzing: self.analyzer.analyzing,
                compiled_scene_ids: self.analyzer.compiled_scenes.keys().map(|s| s.to_string()).collect(),
                has_cfg: self.analyzer.cfg.is_some(),
            },
            debugger: DebuggerSnapshot {
                active: self.debugger.active,
                scene_id: self.debugger.scene_id.as_ref().map(|s| s.to_string()),
                passage_index: self.debugger.passage_index,
                paused: self.debugger.paused,
                breakpoints: self.debugger.breakpoints.iter().map(|(s, i)| (s.to_string(), *i)).collect(),
            },
            player: PlayerSnapshot {
                active: self.player.active,
                scene_id: self.player.scene_id.as_ref().map(|s| s.to_string()),
                passage_index: self.player.passage_index,
                passage_text: self.player.passage_text.clone(),
                choices: self.player.choices.iter().map(|c| PlayerChoiceSnapshot {
                    index: c.index,
                    text: c.text.clone(),
                    enabled: c.enabled,
                    disabled_reason: c.disabled_reason.clone(),
                }).collect(),
            },
            full_player: self.full_player.snapshot(),
        }
    }
}
