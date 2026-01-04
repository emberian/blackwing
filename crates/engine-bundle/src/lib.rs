//! Game bundles for the generalized game engine.
//!
//! A `GameBundle` is a self-contained package that includes:
//! - A manifest describing the game
//! - A schema defining resources, entity types, and systems
//! - Content (entity templates, dialogues, quests)
//!
//! Bundles can be loaded from a directory (for development) or from
//! serialized bytes (for distribution/WASM).

mod error;
mod manifest;
mod schema;

pub use error::BundleError;
pub use manifest::BundleManifest;
pub use schema::{BundleSchema, ComponentDef, EntityTypeDef, InitialStateDef, ResourceDef};

use engine_content::{DialogueTree, EntityTemplate, QuestDef};
use engine_primitives::EntityId;
use engine_runtime::GeneralizedRuntime;
use engine_systems::builtin;
use engine_world::WorldState;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::path::Path;

/// A complete game bundle ready to be played.
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
        }
    }

    /// Load a bundle from a directory.
    ///
    /// Expected structure:
    /// ```text
    /// bundle_dir/
    /// ├── manifest.toml
    /// ├── schema.toml
    /// └── content/
    ///     ├── entities/
    ///     ├── dialogues/
    ///     └── quests/
    /// ```
    pub fn load_from_dir(path: &Path) -> Result<Self, BundleError> {
        // Load manifest
        let manifest_path = path.join("manifest.toml");
        let manifest_str = std::fs::read_to_string(&manifest_path)
            .map_err(|e| BundleError::Io(format!("Failed to read manifest: {}", e)))?;
        let manifest: BundleManifest = toml::from_str(&manifest_str)
            .map_err(|e| BundleError::Parse(format!("Failed to parse manifest: {}", e)))?;

        // Load schema
        let schema_path = path.join("schema.toml");
        let schema = if schema_path.exists() {
            let schema_str = std::fs::read_to_string(&schema_path)
                .map_err(|e| BundleError::Io(format!("Failed to read schema: {}", e)))?;
            toml::from_str(&schema_str)
                .map_err(|e| BundleError::Parse(format!("Failed to parse schema: {}", e)))?
        } else {
            BundleSchema::default()
        };

        // Content loading would go here
        // For now, return an empty bundle with the manifest/schema
        Ok(Self {
            manifest,
            schema,
            templates: Vec::new(),
            dialogues: Vec::new(),
            quests: Vec::new(),
        })
    }

    /// Load a bundle from serialized bytes.
    #[cfg(feature = "binary")]
    pub fn load_from_bytes(data: &[u8]) -> Result<Self, BundleError> {
        bincode::deserialize(data)
            .map_err(|e| BundleError::Parse(format!("Failed to deserialize bundle: {}", e)))
    }

    /// Serialize the bundle to bytes.
    #[cfg(feature = "binary")]
    pub fn to_bytes(&self) -> Result<Vec<u8>, BundleError> {
        bincode::serialize(self)
            .map_err(|e| BundleError::Serialize(format!("Failed to serialize bundle: {}", e)))
    }

    /// Create a runtime configured for this bundle.
    pub fn create_runtime(&self, seed: u64) -> GeneralizedRuntime {
        // Create initial state from schema
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
            if let Some(template) = self.templates.iter().find(|t| &t.id == template_id) {
                let key = state.entities.spawn(template.id.clone());
                for (comp, val) in &template.components {
                    state.entities.set_component(key, comp.clone(), val.clone());
                }
                for tag in &template.tags {
                    state.entities.add_tag(key, tag.clone());
                }
            }
        }

        // Create runtime with state
        let mut runtime = GeneralizedRuntime::with_state(state, seed);

        // Set game mode from manifest
        runtime.set_mode(self.manifest.game_mode.clone());

        // Register all built-in systems
        builtin::register_all(runtime.systems_mut());

        runtime
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

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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
    }

    #[test]
    fn create_runtime_from_bundle() {
        use engine_primitives::ResourceId;

        let mut bundle = GameBundle::new("Test Game");
        bundle.schema.initial_state.resources.insert("gold".into(), 100);

        let runtime = bundle.create_runtime(12345);
        let gold_id: ResourceId = "gold".into();
        assert_eq!(runtime.state().resources.get(&gold_id).copied(), Some(100));
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
}
