//! UI components for the Holdsmith IDE.

mod analyzer;
mod browser;
mod debugger;
mod editor;
mod player;
mod status_bar;
mod toolbar;

pub use analyzer::AnalyzerPanel;
pub use browser::BrowserPanel;
pub use debugger::DebuggerPanel;
pub use editor::EditorPanel;
pub use player::PlayerPanel;
pub use status_bar::StatusBar;
pub use toolbar::Toolbar;
