//! UI components for the Holdsmith IDE.

mod analyzer;
mod browser;
mod debugger;
mod editor;
mod full_player;
mod player;
mod status_bar;
mod toolbar;

pub use analyzer::AnalyzerPanel;
pub use browser::BrowserPanel;
pub use debugger::DebuggerPanel;
pub use editor::EditorPanel;
pub use full_player::FullPlayerPanel;
pub use player::PlayerPanel;
pub use status_bar::StatusBar;
pub use toolbar::Toolbar;
