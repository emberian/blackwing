//! Game bundle - a self-contained game package.

use crate::content::{DialogueTree, EntityTemplate, QuestDef};
use crate::error::BundleError;
use crate::manifest::BundleManifest;
use crate::schema::BundleSchema;
use blackwing_core::{EntityId, ScriptSystem, WorldState};
use blackwing_tilemap::{TileSet, WorldMap};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A complete game bundle ready to be played.
///
/// Contains all content, scripts, and metadata needed to run a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBundle {
    /// Bundle metadata
    pub manifest: BundleManifest,

    /// World schema defining the game's structure
    pub schema: BundleSchema,

    /// Entity templates
    pub templates: Vec<EntityTemplate>,

    /// Dialogue trees
    pub dialogues: Vec<DialogueTree>,

    /// Quest definitions
    pub quests: Vec<QuestDef>,

    /// Script system sources (id -> source code)
    #[serde(default)]
    pub script_systems: Vec<ScriptSource>,

    /// Init script source (run when game starts)
    #[serde(default)]
    pub init_script: Option<String>,

    // === Tilemap content (for Zelda-style games) ===
    /// Tilesets defining tile graphics and collision
    #[serde(default)]
    pub tilesets: Vec<TileSet>,

    /// World maps (overworld, dungeons, etc.)
    #[serde(default)]
    pub worlds: Vec<WorldMap>,

    /// Which world to start in (if using tilemap-based gameplay)
    #[serde(default)]
    pub starting_world: Option<SmolStr>,
}

/// Source code for a script system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptSource {
    /// Script identifier (filename without extension)
    pub id: SmolStr,
    /// Rhai source code
    pub source: String,
}

impl GameBundle {
    /// Create a new empty bundle with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            manifest: BundleManifest::new(name),
            schema: BundleSchema::default(),
            templates: Vec::new(),
            dialogues: Vec::new(),
            quests: Vec::new(),
            script_systems: Vec::new(),
            init_script: None,
            tilesets: Vec::new(),
            worlds: Vec::new(),
            starting_world: None,
        }
    }

    /// Compile all script systems in this bundle.
    ///
    /// Returns a vector of compiled ScriptSystem instances ready for registration.
    pub fn compile_systems(&self) -> Result<Vec<ScriptSystem>, BundleError> {
        let mut systems = Vec::new();
        for script in &self.script_systems {
            let system = ScriptSystem::from_source(&script.source)
                .map_err(|e| BundleError::Script(format!("{}: {}", script.id, e)))?;
            systems.push(system);
        }
        Ok(systems)
    }

    /// Create initial world state from the schema.
    pub fn create_initial_state(&self) -> WorldState {
        let mut state = WorldState::new();

        // Apply initial resources
        for (id, value) in &self.schema.initial_state.resources {
            state.resources.insert(id.clone(), *value);
        }

        // Apply initial flags
        for (id, value) in &self.schema.initial_state.flags {
            state.flags.insert(id.clone(), value.clone());
        }

        // Spawn initial entities from templates
        for template_id in &self.schema.initial_state.spawn_entities {
            if let Some(template) = self.get_template(template_id) {
                let key = state.entities.spawn(template.id.clone());
                for (comp, val) in &template.components {
                    state.entities.set_component(key, comp.clone(), val.clone());
                }
                for tag in &template.tags {
                    state.entities.add_tag(key, tag.clone());
                }
            }
        }

        state
    }

    /// Get an entity template by ID.
    pub fn get_template(&self, id: &EntityId) -> Option<&EntityTemplate> {
        self.templates.iter().find(|t| &t.id == id)
    }

    /// Get a dialogue tree by ID.
    pub fn get_dialogue(&self, id: &SmolStr) -> Option<&DialogueTree> {
        self.dialogues.iter().find(|d| &d.id == id)
    }

    /// Get a quest definition by ID.
    pub fn get_quest(&self, id: &SmolStr) -> Option<&QuestDef> {
        self.quests.iter().find(|q| &q.id == id)
    }

    /// Add an entity template to the bundle.
    pub fn add_template(&mut self, template: EntityTemplate) {
        self.templates.push(template);
    }

    /// Add a dialogue tree to the bundle.
    pub fn add_dialogue(&mut self, dialogue: DialogueTree) {
        self.dialogues.push(dialogue);
    }

    /// Add a quest definition to the bundle.
    pub fn add_quest(&mut self, quest: QuestDef) {
        self.quests.push(quest);
    }

    /// Add a script system source to the bundle.
    pub fn add_script_system(&mut self, id: impl Into<SmolStr>, source: impl Into<String>) {
        self.script_systems.push(ScriptSource {
            id: id.into(),
            source: source.into(),
        });
    }

    /// Set the init script.
    pub fn set_init_script(&mut self, source: impl Into<String>) {
        self.init_script = Some(source.into());
    }

    // === Tilemap methods ===

    /// Add a tileset to the bundle.
    pub fn add_tileset(&mut self, tileset: TileSet) {
        self.tilesets.push(tileset);
    }

    /// Get a tileset by ID.
    pub fn get_tileset(&self, id: &str) -> Option<&TileSet> {
        self.tilesets.iter().find(|t| t.id == id)
    }

    /// Add a world map to the bundle.
    pub fn add_world(&mut self, world: WorldMap) {
        self.worlds.push(world);
    }

    /// Get a world map by ID.
    pub fn get_world(&self, id: &str) -> Option<&WorldMap> {
        self.worlds.iter().find(|w| w.id == id)
    }

    /// Get the starting world map.
    pub fn get_starting_world(&self) -> Option<&WorldMap> {
        self.starting_world
            .as_ref()
            .and_then(|id| self.get_world(id))
    }

    /// Set the starting world.
    pub fn set_starting_world(&mut self, world_id: impl Into<SmolStr>) {
        self.starting_world = Some(world_id.into());
    }

    /// Validate the bundle for consistency.
    pub fn validate(&self) -> Result<(), Vec<BundleError>> {
        let mut errors = Vec::new();

        // Check entry dialogue exists if specified
        if let Some(entry) = &self.manifest.entry_dialogue {
            if self.get_dialogue(entry).is_none() {
                errors.push(BundleError::Validation(format!(
                    "Entry dialogue '{}' not found",
                    entry
                )));
            }
        }

        // Check initial location exists if specified
        if let Some(loc) = &self.manifest.initial_location {
            if self.get_template(loc).is_none() {
                errors.push(BundleError::Validation(format!(
                    "Initial location '{}' not found",
                    loc.as_qualified()
                )));
            }
        }

        // Check spawn_entities exist
        for template_id in &self.schema.initial_state.spawn_entities {
            if self.get_template(template_id).is_none() {
                errors.push(BundleError::Validation(format!(
                    "Spawn entity template '{}' not found",
                    template_id.as_qualified()
                )));
            }
        }

        // Validate script systems compile
        for script in &self.script_systems {
            if let Err(e) = ScriptSystem::from_source(&script.source) {
                errors.push(BundleError::Script(format!(
                    "System '{}' failed to compile: {}",
                    script.id, e
                )));
            }
        }

        // === Tilemap validation ===

        // Check starting world exists if specified
        if let Some(world_id) = &self.starting_world {
            if self.get_world(world_id).is_none() {
                errors.push(BundleError::Validation(format!(
                    "Starting world '{}' not found",
                    world_id
                )));
            }
        }

        // Validate each world
        for world in &self.worlds {
            // Check world's tileset references exist
            for room in world.rooms.values() {
                if self.get_tileset(&room.tileset_id).is_none() {
                    errors.push(BundleError::Validation(format!(
                        "Room '{}' in world '{}' references non-existent tileset '{}'",
                        room.id, world.id, room.tileset_id
                    )));
                }
            }

            // Validate world internal consistency
            for error in world.validate() {
                errors.push(BundleError::Validation(format!(
                    "World '{}': {}",
                    world.id, error
                )));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Serialize the bundle to bytes.
    #[cfg(feature = "binary")]
    pub fn to_bytes(&self) -> Result<Vec<u8>, BundleError> {
        bincode::serialize(self)
            .map_err(|e| BundleError::Serialize(format!("Failed to serialize bundle: {}", e)))
    }

    /// Load a bundle from serialized bytes.
    #[cfg(feature = "binary")]
    pub fn from_bytes(data: &[u8]) -> Result<Self, BundleError> {
        bincode::deserialize(data)
            .map_err(|e| BundleError::Parse(format!("Failed to deserialize bundle: {}", e)))
    }
}

impl Default for GameBundle {
    fn default() -> Self {
        Self::new("Untitled Game")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_empty_bundle() {
        let bundle = GameBundle::new("Test Game");
        assert_eq!(bundle.manifest.name, "Test Game");
        assert!(bundle.templates.is_empty());
        assert!(bundle.dialogues.is_empty());
        assert!(bundle.script_systems.is_empty());
    }

    #[test]
    fn validate_empty_bundle() {
        let bundle = GameBundle::new("Test Game");
        assert!(bundle.validate().is_ok());
    }

    #[test]
    fn validate_missing_entry_dialogue() {
        let mut bundle = GameBundle::new("Test Game");
        bundle.manifest.entry_dialogue = Some("nonexistent".into());

        let result = bundle.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn add_script_system() {
        let mut bundle = GameBundle::new("Test Game");
        bundle.add_script_system(
            "test_system",
            r#"
const SYSTEM_ID = "test";
const HANDLES = ["test_command"];

fn handle_command(cmd) {
    chronicle("Test", "Handled");
}
"#,
        );

        assert_eq!(bundle.script_systems.len(), 1);
        assert_eq!(bundle.script_systems[0].id, "test_system");
    }

    #[test]
    fn compile_script_systems() {
        use blackwing_core::System;

        let mut bundle = GameBundle::new("Test Game");
        bundle.add_script_system(
            "test_system",
            r#"
const SYSTEM_ID = "test";
const HANDLES = ["test_command"];

fn handle_command(cmd) {
    chronicle("Test", "Handled");
}
"#,
        );

        let systems = bundle.compile_systems().unwrap();
        assert_eq!(systems.len(), 1);
        assert_eq!(systems[0].id(), "test");
    }

    #[test]
    fn create_initial_state() {
        use blackwing_core::ResourceId;

        let mut bundle = GameBundle::new("Test Game");
        bundle.schema.initial_state.resources.insert("gold".into(), 100);

        let state = bundle.create_initial_state();
        let gold_id: ResourceId = "gold".into();
        assert_eq!(state.resources.get(&gold_id).copied(), Some(100));
    }
}
