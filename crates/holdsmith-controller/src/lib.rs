//! Holdsmith IDE controller.
//!
//! This crate provides state management and command dispatch for the Holdsmith IDE:
//! - Project state (virtual filesystem, open files)
//! - Editor state (content, cursor, diagnostics)
//! - Analyzer state (CFG, analysis results)
//! - Debugger state (runtime, breakpoints)
//! - Player state (scene playback)

mod commands;
mod state;
mod view_models;

pub use commands::{Command, CommandResult};
pub use state::{
    AnalyzerSnapshot, AnalyzerState, AppState, AppStateSnapshot, DebugLocation, DebuggerSnapshot,
    DebuggerState, EditorSnapshot, EditorState, PlayerChoiceSnapshot, PlayerSnapshot, PlayerState,
    ProjectSnapshot, ProjectState,
};
pub use view_models::{
    ChoiceViewModel, DiagnosticViewModel, FileTreeNode, LocationViewModel, PassageViewModel,
    SeverityViewModel, StateInspector,
};

use vfs::{MemoryFS, VfsPath};

/// The main controller for the Holdsmith IDE.
pub struct Controller {
    state: AppState,
    listeners: Vec<Box<dyn Fn(&AppState) + Send + Sync>>,
}

impl Controller {
    /// Create a new controller with an in-memory filesystem.
    pub fn new() -> Self {
        let fs: VfsPath = MemoryFS::new().into();
        Self {
            state: AppState::new(fs),
            listeners: Vec::new(),
        }
    }

    /// Create a new controller with a custom filesystem.
    pub fn with_fs(fs: VfsPath) -> Self {
        Self {
            state: AppState::new(fs),
            listeners: Vec::new(),
        }
    }

    /// Get the current application state.
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Subscribe to state changes.
    pub fn subscribe(&mut self, listener: impl Fn(&AppState) + Send + Sync + 'static) {
        self.listeners.push(Box::new(listener));
    }

    /// Dispatch a command and return the result.
    pub fn dispatch(&mut self, cmd: Command) -> CommandResult {
        let result = commands::dispatch(&mut self.state, cmd);
        if result.state_changed {
            self.notify();
        }
        result
    }

    /// Notify all listeners of state change.
    fn notify(&self) {
        for listener in &self.listeners {
            listener(&self.state);
        }
    }

    // === Convenience methods ===

    /// Open a file in the editor.
    pub fn open_file(&mut self, path: &str) -> CommandResult {
        self.dispatch(Command::OpenFile {
            path: path.to_string(),
        })
    }

    /// Save the current file.
    pub fn save_current(&mut self) -> CommandResult {
        self.dispatch(Command::SaveCurrentFile)
    }

    /// Update editor content.
    pub fn set_content(&mut self, content: String) -> CommandResult {
        self.dispatch(Command::SetContent { content })
    }

    /// Analyze the current file.
    pub fn analyze_current(&mut self) -> CommandResult {
        self.dispatch(Command::AnalyzeCurrentFile)
    }

    /// Start playing a scene.
    pub fn start_play(&mut self, scene_id: &str) -> CommandResult {
        self.dispatch(Command::StartPlay {
            scene_id: scene_id.to_string(),
        })
    }

    /// Make a choice in the player.
    pub fn make_choice(&mut self, index: usize) -> CommandResult {
        self.dispatch(Command::MakeChoice { index })
    }

    /// Create a serializable snapshot of the current state.
    pub fn snapshot(&self) -> AppStateSnapshot {
        self.state.snapshot()
    }

    /// Export the project as a zip file.
    pub fn export_zip(&self) -> Result<Vec<u8>, ExportError> {
        export_project(&self.state.project.fs)
    }

    /// Import a project from a zip file.
    pub fn import_zip(&mut self, data: &[u8]) -> Result<(), ImportError> {
        import_project(&self.state.project.fs, data)?;
        self.state.project.refresh_file_tree();
        self.notify();
        Ok(())
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

/// Error during project export.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("VFS error: {0}")]
    Vfs(String),
}

/// Error during project import.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("VFS error: {0}")]
    Vfs(String),
}

/// Export a VFS to a zip file.
fn export_project(fs: &VfsPath) -> Result<Vec<u8>, ExportError> {
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buffer);
    let options = SimpleFileOptions::default();

    fn add_dir(
        zip: &mut ZipWriter<&mut Cursor<Vec<u8>>>,
        path: &VfsPath,
        options: SimpleFileOptions,
    ) -> Result<(), ExportError> {
        for entry in path.read_dir().map_err(|e| ExportError::Vfs(e.to_string()))? {
            if entry
                .is_dir()
                .map_err(|e| ExportError::Vfs(e.to_string()))?
            {
                add_dir(zip, &entry, options)?;
            } else {
                let content = entry
                    .read_to_string()
                    .map_err(|e| ExportError::Vfs(e.to_string()))?;
                let relative = entry.as_str().trim_start_matches('/');
                zip.start_file(relative, options)?;
                zip.write_all(content.as_bytes())?;
            }
        }
        Ok(())
    }

    add_dir(&mut zip, fs, options)?;
    let _ = zip.finish()?;
    Ok(buffer.into_inner())
}

/// Import a zip file into a VFS.
fn import_project(fs: &VfsPath, data: &[u8]) -> Result<(), ImportError> {
    use std::io::{Cursor, Read};
    use zip::ZipArchive;

    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if file.is_dir() {
            fs.join(&name)
                .map_err(|e| ImportError::Vfs(e.to_string()))?
                .create_dir_all()
                .map_err(|e| ImportError::Vfs(e.to_string()))?;
        } else {
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            let path = fs
                .join(&name)
                .map_err(|e| ImportError::Vfs(e.to_string()))?;

            // Ensure parent directory exists
            let parent = path.parent();
            parent
                .create_dir_all()
                .map_err(|e| ImportError::Vfs(e.to_string()))?;

            path.create_file()
                .map_err(|e| ImportError::Vfs(e.to_string()))?
                .write_all(content.as_bytes())
                .map_err(|e| ImportError::Vfs(e.to_string()))?;
        }
    }

    Ok(())
}
