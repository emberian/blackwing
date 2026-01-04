//! Bundle manifest defining game metadata.

use blackwing_core::{EntityId, GameMode};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Metadata about a game bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    /// Name of the game
    pub name: String,

    /// Version string (semantic versioning recommended)
    #[serde(default = "default_version")]
    pub version: String,

    /// Author or studio name
    #[serde(default)]
    pub author: Option<String>,

    /// Short description of the game
    #[serde(default)]
    pub description: Option<String>,

    /// Entry dialogue ID (the first dialogue/conversation to play)
    #[serde(default)]
    pub entry_dialogue: Option<SmolStr>,

    /// Initial location for the player (for MUD-style games)
    #[serde(default)]
    pub initial_location: Option<EntityId>,

    /// Game mode (turn-based, real-time, hybrid)
    #[serde(default)]
    pub game_mode: GameMode,

    /// Init script to run when the game starts
    #[serde(default)]
    pub init_script: Option<SmolStr>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

impl BundleManifest {
    /// Create a new manifest with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: default_version(),
            author: None,
            description: None,
            entry_dialogue: None,
            initial_location: None,
            game_mode: GameMode::TurnBased,
            init_script: None,
        }
    }

    /// Set the version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set the author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the entry dialogue.
    pub fn with_entry_dialogue(mut self, dialogue_id: impl Into<SmolStr>) -> Self {
        self.entry_dialogue = Some(dialogue_id.into());
        self
    }

    /// Set the initial location.
    pub fn with_initial_location(mut self, location: EntityId) -> Self {
        self.initial_location = Some(location);
        self
    }

    /// Set the game mode.
    pub fn with_game_mode(mut self, mode: GameMode) -> Self {
        self.game_mode = mode;
        self
    }

    /// Set the init script.
    pub fn with_init_script(mut self, script: impl Into<SmolStr>) -> Self {
        self.init_script = Some(script.into());
        self
    }
}

impl Default for BundleManifest {
    fn default() -> Self {
        Self::new("Untitled Game")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_manifest() {
        let manifest = BundleManifest::new("My Game")
            .with_version("1.0.0")
            .with_author("Test Author")
            .with_description("A test game");

        assert_eq!(manifest.name, "My Game");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.author.as_deref(), Some("Test Author"));
    }

    #[test]
    fn serialize_manifest() {
        let manifest = BundleManifest::new("Test Game");
        let toml_str = toml::to_string(&manifest).unwrap();
        assert!(toml_str.contains("name = \"Test Game\""));
    }

    #[test]
    fn deserialize_manifest() {
        let toml_str = r#"
name = "My Adventure"
version = "2.0.0"
author = "Jane Doe"
"#;
        let manifest: BundleManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.name, "My Adventure");
        assert_eq!(manifest.version, "2.0.0");
        assert_eq!(manifest.author.as_deref(), Some("Jane Doe"));
    }
}
