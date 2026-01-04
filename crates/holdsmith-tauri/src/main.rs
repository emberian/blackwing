//! Holdsmith IDE - Native Desktop Application
//!
//! This is the Tauri wrapper that provides:
//! - Native filesystem access via PhysicalFS
//! - Full Z3 symbolic analysis
//! - Desktop window management

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use holdsmith_controller::{AppStateSnapshot, Command, CommandResult, Controller};
use tauri::State;
use vfs::{PhysicalFS, VfsPath};

/// Application state shared across all commands.
struct AppState {
    controller: Mutex<Controller>,
    project_root: Mutex<Option<String>>,
}

/// Open a project directory.
#[tauri::command]
fn open_project(path: String, state: State<AppState>) -> Result<(), String> {
    let fs: VfsPath = PhysicalFS::new(&path).into();
    let controller = Controller::with_fs(fs);

    *state.controller.lock().map_err(|e| e.to_string())? = controller;
    *state.project_root.lock().map_err(|e| e.to_string())? = Some(path);

    Ok(())
}

/// Get the file tree for the current project.
#[tauri::command]
fn get_file_tree(state: State<AppState>) -> Result<String, String> {
    let ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    let tree = &ctrl.state().project.file_tree;
    serde_json::to_string(tree).map_err(|e| e.to_string())
}

/// Read a file from the project.
#[tauri::command]
fn open_file(path: String, state: State<AppState>) -> Result<String, String> {
    let mut ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    ctrl.open_file(&path);
    Ok(ctrl.state().editor.content.clone())
}

/// Write content to a file.
#[tauri::command]
fn save_file(state: State<AppState>) -> Result<(), String> {
    let mut ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    ctrl.save_current();
    Ok(())
}

/// Update the editor content.
#[tauri::command]
fn set_content(content: String, state: State<AppState>) -> Result<(), String> {
    let mut ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    ctrl.set_content(content);
    Ok(())
}

/// Get the current diagnostics as JSON.
#[tauri::command]
fn get_diagnostics(state: State<AppState>) -> Result<String, String> {
    let ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    let diagnostics: Vec<_> = ctrl
        .state()
        .editor
        .diagnostics
        .iter()
        .map(holdsmith_controller::DiagnosticViewModel::from)
        .collect();
    serde_json::to_string(&diagnostics).map_err(|e| e.to_string())
}

/// Analyze the current file with Z3 (native only).
#[tauri::command]
fn analyze_current(state: State<AppState>) -> Result<String, String> {
    let mut ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    ctrl.analyze_current();

    // Return whether CFG is available and diagnostic count
    let has_cfg = ctrl.state().analyzer.cfg.is_some();
    let diagnostic_count = ctrl.state().editor.diagnostics.len();
    Ok(serde_json::json!({
        "has_cfg": has_cfg,
        "diagnostic_count": diagnostic_count,
        "analyzing": ctrl.state().analyzer.analyzing
    }).to_string())
}

/// Create a new file.
#[tauri::command]
fn new_file(path: String, state: State<AppState>) -> Result<(), String> {
    let mut ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    ctrl.dispatch(holdsmith_controller::Command::NewFile { path });
    Ok(())
}

/// Start playing a scene.
#[tauri::command]
fn start_play(scene_id: String, state: State<AppState>) -> Result<String, String> {
    let mut ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    ctrl.start_play(&scene_id);

    // Return current player state as JSON
    let player = &ctrl.state().player;
    Ok(serde_json::json!({
        "passage_index": player.passage_index,
        "passage_text": player.passage_text.clone(),
        "active": player.active,
        "choices": player.choices.iter().map(|c| {
            serde_json::json!({
                "index": c.index,
                "text": c.text.clone(),
                "enabled": c.enabled,
            })
        }).collect::<Vec<_>>(),
    }).to_string())
}

/// Make a choice in the player.
#[tauri::command]
fn make_choice(index: usize, state: State<AppState>) -> Result<String, String> {
    let mut ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    ctrl.make_choice(index);

    let player = &ctrl.state().player;
    Ok(serde_json::json!({
        "passage_index": player.passage_index,
        "passage_text": player.passage_text.clone(),
        "active": player.active,
        "choices": player.choices.iter().map(|c| {
            serde_json::json!({
                "index": c.index,
                "text": c.text.clone(),
                "enabled": c.enabled,
            })
        }).collect::<Vec<_>>(),
    }).to_string())
}

/// Get app info for the frontend.
#[tauri::command]
fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "Holdsmith IDE",
        "version": env!("CARGO_PKG_VERSION"),
        "z3_enabled": true,
        "platform": "native"
    })
}

// ============================================================================
// Generic IPC Commands (for Backend abstraction)
// ============================================================================

/// Get a complete snapshot of the application state.
#[tauri::command]
fn get_snapshot(state: State<AppState>) -> Result<AppStateSnapshot, String> {
    let ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    Ok(ctrl.snapshot())
}

/// Dispatch any command to the controller.
#[tauri::command]
fn dispatch_command(cmd: Command, state: State<AppState>) -> Result<CommandResult, String> {
    let mut ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    Ok(ctrl.dispatch(cmd))
}

/// Export the project as a zip file.
#[tauri::command]
fn export_zip(state: State<AppState>) -> Result<Vec<u8>, String> {
    let ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    ctrl.export_zip().map_err(|e| e.to_string())
}

/// Import a project from a zip file.
#[tauri::command]
fn import_zip(data: Vec<u8>, state: State<AppState>) -> Result<(), String> {
    let mut ctrl = state.controller.lock().map_err(|e| e.to_string())?;
    ctrl.import_zip(&data).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            controller: Mutex::new(Controller::new()),
            project_root: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            // Legacy commands (kept for backwards compatibility)
            open_project,
            get_file_tree,
            open_file,
            save_file,
            set_content,
            get_diagnostics,
            analyze_current,
            new_file,
            start_play,
            make_choice,
            get_app_info,
            // Generic IPC commands (for Backend abstraction)
            get_snapshot,
            dispatch_command,
            export_zip,
            import_zip,
        ])
        .run(tauri::generate_context!())
        .expect("error running Holdsmith");
}
