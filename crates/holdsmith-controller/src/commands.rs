//! Command dispatch and handling.

use engine_core::Navigation;
use holdsmith_analyzer::{analyze_scene, build_cfg, generate_diagnostics};
use holdsmith_compiler::compile;
use holdsmith_parser::parse;
use smol_str::SmolStr;

use crate::state::{AppState, PlayerChoice, PlayerHistoryEntry};

/// Commands that can be dispatched to the controller.
#[derive(Debug, Clone)]
pub enum Command {
    // === Project commands ===
    /// Create a new project (clears filesystem).
    NewProject,

    // === File commands ===
    /// Create a new file.
    NewFile { path: String },
    /// Open a file in the editor.
    OpenFile { path: String },
    /// Close a file.
    CloseFile { path: String },
    /// Save the current file.
    SaveCurrentFile,
    /// Save a specific file.
    SaveFile { path: String },
    /// Save all dirty files.
    SaveAll,
    /// Delete a file.
    DeleteFile { path: String },
    /// Rename a file.
    RenameFile { old_path: String, new_path: String },

    // === Editor commands ===
    /// Set the editor content (triggers re-parse).
    SetContent { content: String },
    /// Set cursor position.
    SetCursor { line: usize, column: usize },
    /// Set selection range.
    SetSelection { start: usize, end: usize },
    /// Clear selection.
    ClearSelection,

    // === Analyzer commands ===
    /// Analyze the current file.
    AnalyzeCurrentFile,
    /// Analyze a specific file.
    AnalyzeFile { path: String },
    /// Analyze all scene files in the project.
    AnalyzeProject,

    // === Debugger commands ===
    /// Start debugging a scene.
    StartDebug { scene_id: String },
    /// Stop debugging.
    StopDebug,
    /// Set a breakpoint.
    SetBreakpoint { scene_id: String, passage_index: usize },
    /// Remove a breakpoint.
    RemoveBreakpoint { scene_id: String, passage_index: usize },
    /// Step into (advance to next passage).
    StepInto,
    /// Continue execution until breakpoint or end.
    Continue,

    // === Player commands ===
    /// Start playing a scene.
    StartPlay { scene_id: String },
    /// Stop playing.
    StopPlay,
    /// Make a choice.
    MakeChoice { index: usize },
    /// Restart the current scene.
    RestartScene,
}

/// Result of dispatching a command.
#[derive(Debug)]
pub struct CommandResult {
    /// Whether the command succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
    /// Whether state changed (triggers UI update).
    pub state_changed: bool,
}

impl CommandResult {
    fn ok() -> Self {
        Self {
            success: true,
            error: None,
            state_changed: true,
        }
    }

    fn ok_no_change() -> Self {
        Self {
            success: true,
            error: None,
            state_changed: false,
        }
    }

    fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(msg.into()),
            state_changed: false,
        }
    }
}

/// Dispatch a command and update state.
pub fn dispatch(state: &mut AppState, cmd: Command) -> CommandResult {
    match cmd {
        // === Project ===
        Command::NewProject => {
            // Clear state
            state.project.open_files.clear();
            state.project.active_file = None;
            state.project.dirty_files.clear();
            state.editor = Default::default();
            state.analyzer = Default::default();
            state.debugger = Default::default();
            state.player = Default::default();
            state.project.refresh_file_tree();
            CommandResult::ok()
        }

        // === Files ===
        Command::NewFile { path } => {
            if state.project.file_exists(&path) {
                return CommandResult::err(format!("File already exists: {}", path));
            }
            if let Err(e) = state.project.write_file(&path, "") {
                return CommandResult::err(e);
            }
            // Open the new file
            dispatch(state, Command::OpenFile { path })
        }

        Command::OpenFile { path } => {
            let content = match state.project.read_file(&path) {
                Some(c) => c,
                None => return CommandResult::err(format!("File not found: {}", path)),
            };

            // Add to open files if not already open
            if !state.project.open_files.contains(&path) {
                state.project.open_files.push(path.clone());
            }

            state.project.active_file = Some(path.clone());
            state.editor.content = content.clone();
            state.editor.cursor = (0, 0);
            state.editor.selection = None;

            // Parse the file
            parse_current_file(state, &path);

            // Run analysis
            analyze_current_file(state, &path);

            CommandResult::ok()
        }

        Command::CloseFile { path } => {
            state.project.open_files.retain(|p| p != &path);
            state.project.dirty_files.remove(&path);

            if state.project.active_file.as_ref() == Some(&path) {
                // Switch to another open file, or clear editor
                state.project.active_file = state.project.open_files.first().cloned();
                if let Some(new_path) = state.project.active_file.clone() {
                    if let Some(content) = state.project.read_file(&new_path) {
                        state.editor.content = content;
                        parse_current_file(state, &new_path);
                        analyze_current_file(state, &new_path);
                    }
                } else {
                    state.editor = Default::default();
                }
            }
            CommandResult::ok()
        }

        Command::SaveCurrentFile => {
            let path = match &state.project.active_file {
                Some(p) => p.clone(),
                None => return CommandResult::err("No file is currently open"),
            };
            dispatch(state, Command::SaveFile { path })
        }

        Command::SaveFile { path } => {
            if let Err(e) = state.project.write_file(&path, &state.editor.content) {
                return CommandResult::err(e);
            }
            state.project.dirty_files.remove(&path);
            CommandResult::ok()
        }

        Command::SaveAll => {
            let dirty: Vec<_> = state.project.dirty_files.iter().cloned().collect();
            for path in dirty {
                // Save the current file if it's dirty
                if state.project.active_file.as_ref() == Some(&path) {
                    if let Err(e) = state.project.write_file(&path, &state.editor.content) {
                        return CommandResult::err(e);
                    }
                }
                state.project.dirty_files.remove(&path);
            }
            CommandResult::ok()
        }

        Command::DeleteFile { path } => {
            // Close if open
            let _ = dispatch(state, Command::CloseFile { path: path.clone() });

            // Delete from filesystem
            if let Ok(file_path) = state.project.fs.join(&path) {
                if let Err(e) = file_path.remove_file() {
                    return CommandResult::err(format!("Failed to delete: {}", e));
                }
            }
            state.project.refresh_file_tree();
            CommandResult::ok()
        }

        Command::RenameFile { old_path, new_path } => {
            // Read content
            let content = match state.project.read_file(&old_path) {
                Some(c) => c,
                None => return CommandResult::err(format!("File not found: {}", old_path)),
            };

            // Write to new location
            if let Err(e) = state.project.write_file(&new_path, &content) {
                return CommandResult::err(e);
            }

            // Delete old file
            if let Ok(file_path) = state.project.fs.join(&old_path) {
                let _ = file_path.remove_file();
            }

            // Update open files
            if let Some(idx) = state.project.open_files.iter().position(|p| p == &old_path) {
                state.project.open_files[idx] = new_path.clone();
            }
            if state.project.active_file.as_ref() == Some(&old_path) {
                state.project.active_file = Some(new_path);
            }

            state.project.dirty_files.remove(&old_path);
            state.project.refresh_file_tree();
            CommandResult::ok()
        }

        // === Editor ===
        Command::SetContent { content } => {
            let changed = state.editor.content != content;
            state.editor.content = content;

            if changed {
                // Mark as dirty
                if let Some(path) = state.project.active_file.clone() {
                    state.project.dirty_files.insert(path.clone());
                    // Re-parse
                    parse_current_file(state, &path);
                    // Re-analyze
                    analyze_current_file(state, &path);
                }
            }

            if changed {
                CommandResult::ok()
            } else {
                CommandResult::ok_no_change()
            }
        }

        Command::SetCursor { line, column } => {
            state.editor.cursor = (line, column);
            CommandResult::ok()
        }

        Command::SetSelection { start, end } => {
            state.editor.selection = Some((start, end));
            CommandResult::ok()
        }

        Command::ClearSelection => {
            state.editor.selection = None;
            CommandResult::ok()
        }

        // === Analyzer ===
        Command::AnalyzeCurrentFile => {
            if let Some(ref path) = state.project.active_file.clone() {
                analyze_current_file(state, path);
            }
            CommandResult::ok()
        }

        Command::AnalyzeFile { path } => {
            if let Some(content) = state.project.read_file(&path) {
                analyze_content(state, &content, &path);
            }
            CommandResult::ok()
        }

        Command::AnalyzeProject => {
            // Find all .scene files and analyze them
            state.analyzer.analyzing = true;
            // TODO: Walk filesystem and analyze all .scene files
            state.analyzer.analyzing = false;
            CommandResult::ok()
        }

        // === Debugger ===
        Command::StartDebug { scene_id } => {
            let scene_id = SmolStr::new(&scene_id);

            // Get compiled scene
            let _scene = match state.analyzer.compiled_scenes.get(&scene_id) {
                Some(s) => s.clone(),
                None => return CommandResult::err("Scene not compiled"),
            };

            state.debugger.active = true;
            state.debugger.scene_id = Some(scene_id);
            state.debugger.passage_index = Some(0);
            state.debugger.paused = true;
            state.debugger.passage_history.clear();
            state.debugger.passage_history.push(0);

            CommandResult::ok()
        }

        Command::StopDebug => {
            state.debugger.active = false;
            state.debugger.scene_id = None;
            state.debugger.passage_index = None;
            state.debugger.paused = false;
            state.debugger.passage_history.clear();
            CommandResult::ok()
        }

        Command::SetBreakpoint {
            scene_id,
            passage_index,
        } => {
            state
                .debugger
                .breakpoints
                .insert((SmolStr::new(&scene_id), passage_index));
            CommandResult::ok()
        }

        Command::RemoveBreakpoint {
            scene_id,
            passage_index,
        } => {
            state
                .debugger
                .breakpoints
                .remove(&(SmolStr::new(&scene_id), passage_index));
            CommandResult::ok()
        }

        Command::StepInto => {
            // TODO: Implement stepping logic
            CommandResult::ok()
        }

        Command::Continue => {
            // TODO: Implement continue logic
            CommandResult::ok()
        }

        // === Player ===
        Command::StartPlay { scene_id } => {
            let scene_id = SmolStr::new(&scene_id);

            // Get compiled scene
            let scene = match state.analyzer.compiled_scenes.get(&scene_id) {
                Some(s) => s.clone(),
                None => return CommandResult::err("Scene not compiled"),
            };

            state.player.active = true;
            state.player.scene_id = Some(scene_id);
            state.player.passage_index = 0;
            state.player.history.clear();

            // Load first passage
            load_passage(state, &scene, 0);

            CommandResult::ok()
        }

        Command::StopPlay => {
            state.player.active = false;
            state.player.scene_id = None;
            state.player.passage_text.clear();
            state.player.choices.clear();
            state.player.history.clear();
            CommandResult::ok()
        }

        Command::MakeChoice { index } => {
            let scene_id = match &state.player.scene_id {
                Some(id) => id.clone(),
                None => return CommandResult::err("No scene is playing"),
            };

            let scene = match state.analyzer.compiled_scenes.get(&scene_id) {
                Some(s) => s.clone(),
                None => return CommandResult::err("Scene not compiled"),
            };

            let passage = match scene.passages.get(state.player.passage_index) {
                Some(p) => p,
                None => return CommandResult::err("Invalid passage index"),
            };

            let choice = match passage.choices.get(index) {
                Some(c) => c,
                None => return CommandResult::err("Invalid choice index"),
            };

            // Record history
            state.player.history.push(PlayerHistoryEntry {
                passage_index: state.player.passage_index,
                choice_index: Some(index),
            });

            // Navigate to next passage
            match &choice.next {
                Navigation::Passage(idx) => {
                    state.player.passage_index = *idx;
                    load_passage(state, &scene, *idx);
                }
                Navigation::End => {
                    state.player.passage_text = "[Scene End]".to_string();
                    state.player.choices.clear();
                }
            }

            CommandResult::ok()
        }

        Command::RestartScene => {
            let scene_id = match &state.player.scene_id {
                Some(id) => id.clone(),
                None => return CommandResult::err("No scene is playing"),
            };
            dispatch(
                state,
                Command::StartPlay {
                    scene_id: scene_id.to_string(),
                },
            )
        }
    }
}

/// Parse the current editor content.
fn parse_current_file(state: &mut AppState, filename: &str) {
    state.editor.parse_errors.clear();
    state.editor.parsed_ast = None;

    match parse(&state.editor.content, filename) {
        Ok(ast) => {
            state.editor.parsed_ast = Some(ast);
        }
        Err(e) => {
            state.editor.parse_errors.push(e.to_string());
        }
    }
}

/// Analyze the current file.
fn analyze_current_file(state: &mut AppState, filename: &str) {
    analyze_content(state, &state.editor.content.clone(), filename);
}

/// Analyze content and update state.
fn analyze_content(state: &mut AppState, content: &str, filename: &str) {
    state.editor.diagnostics.clear();
    state.analyzer.cfg = None;

    // Parse
    let ast = match parse(content, filename) {
        Ok(ast) => ast,
        Err(_) => return, // Parse errors already shown
    };

    // Compile to Scene
    let scene = match compile(ast) {
        Ok(s) => s,
        Err(_) => {
            // Compilation error - could add to diagnostics
            return;
        }
    };

    let scene_id = SmolStr::from(scene.id.as_str());

    // Store compiled scene for player/debugger
    state
        .analyzer
        .compiled_scenes
        .insert(scene_id.clone(), scene.clone());

    // Build CFG
    let cfg = build_cfg(&scene);
    state.analyzer.cfg = Some(cfg);

    // Run analysis
    let result = analyze_scene(&scene);

    // Generate diagnostics
    state.editor.diagnostics = generate_diagnostics(&result);

    // Store result
    state.analyzer.results.insert(scene_id, result);
}

/// Load a passage into the player state.
fn load_passage(state: &mut AppState, scene: &engine_core::Scene, passage_index: usize) {
    let passage = match scene.passages.get(passage_index) {
        Some(p) => p,
        None => {
            state.player.passage_text = "[Invalid passage]".to_string();
            state.player.choices.clear();
            return;
        }
    };

    state.player.passage_text = passage.text.to_string();
    state.player.choices = passage
        .choices
        .iter()
        .enumerate()
        .map(|(i, c)| PlayerChoice {
            index: i,
            text: c.text.to_string(),
            enabled: true, // TODO: Evaluate conditions
            disabled_reason: None,
        })
        .collect();
}
