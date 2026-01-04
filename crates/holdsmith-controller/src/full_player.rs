//! Full game player using the actual engine runtime.
//!
//! Unlike the simple player which just walks through passages,
//! this uses the complete game engine with Rhai script execution,
//! resources, flags, and all game mechanics.

use std::collections::HashMap;

use engine_core::{
    CardDef, CardId, ContentRegistry, Effect, FlagId, GameState, ResourceId, Scene, SceneId,
    TagCategoryId, TagId, TagProvider, Value,
};
use blackwing_core::Rng;
use engine_script::ScriptExecutor;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A content registry backed by holdsmith's compiled scenes.
pub struct HoldsmithContentRegistry {
    scenes: HashMap<SceneId, Scene>,
}

impl HoldsmithContentRegistry {
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
        }
    }

    pub fn add_scene(&mut self, scene: Scene) {
        let id = scene.id.clone();
        self.scenes.insert(id, scene);
    }

    pub fn clear(&mut self) {
        self.scenes.clear();
    }
}

impl ContentRegistry for HoldsmithContentRegistry {
    fn get_scene(&self, id: &SceneId) -> Option<&Scene> {
        self.scenes.get(id)
    }

    fn get_card(&self, _id: &CardId) -> Option<&CardDef> {
        None // No cards in holdsmith preview
    }

    fn scenes(&self) -> Box<dyn Iterator<Item = &Scene> + '_> {
        Box::new(self.scenes.values())
    }

    fn cards(&self) -> Box<dyn Iterator<Item = &CardDef> + '_> {
        Box::new(std::iter::empty())
    }

    fn scene_count(&self) -> usize {
        self.scenes.len()
    }

    fn card_count(&self) -> usize {
        0
    }
}

/// Full player state with actual game engine.
pub struct FullPlayerState {
    /// Whether the full player is active.
    pub active: bool,
    /// Current scene ID.
    pub scene_id: Option<SceneId>,
    /// Current passage index.
    pub passage_index: usize,
    /// Game state (resources, flags, cooldowns).
    pub game_state: GameState,
    /// Content registry with compiled scenes.
    pub content: HoldsmithContentRegistry,
    /// Script executor for Rhai.
    pub script_executor: ScriptExecutor,
    /// RNG for deterministic randomness.
    pub rng: Rng,
    /// Current passage text (after script interpolation).
    pub passage_text: String,
    /// Current choices (with conditions evaluated).
    pub choices: Vec<FullPlayerChoice>,
    /// Chronicle/log entries.
    pub chronicle: Vec<ChronicleEntry>,
    /// Script execution errors (for debugging).
    pub script_errors: Vec<String>,
}

impl std::fmt::Debug for FullPlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FullPlayerState")
            .field("active", &self.active)
            .field("scene_id", &self.scene_id)
            .field("passage_index", &self.passage_index)
            .field("passage_text", &self.passage_text)
            .field("choices", &self.choices.len())
            .field("chronicle", &self.chronicle.len())
            .field("script_errors", &self.script_errors.len())
            .field("resources", &self.game_state.resources.len())
            .field("flags", &self.game_state.flags.len())
            .finish_non_exhaustive()
    }
}

impl Default for FullPlayerState {
    fn default() -> Self {
        Self::new(12345)
    }
}

impl FullPlayerState {
    pub fn new(seed: u64) -> Self {
        Self {
            active: false,
            scene_id: None,
            passage_index: 0,
            game_state: GameState::default(),
            content: HoldsmithContentRegistry::new(),
            script_executor: ScriptExecutor::new(),
            rng: Rng::new(seed),
            passage_text: String::new(),
            choices: Vec::new(),
            chronicle: Vec::new(),
            script_errors: Vec::new(),
        }
    }

    /// Sync compiled scenes from the analyzer.
    pub fn sync_scenes(&mut self, scenes: &HashMap<SmolStr, Scene>) {
        self.content.clear();
        for scene in scenes.values() {
            self.content.add_scene(scene.clone());
        }
    }

    /// Start playing a scene.
    pub fn start(&mut self, scene_id: &str) -> Result<(), String> {
        let scene_id = SceneId::new(scene_id);

        let scene = self.content
            .get_scene(&scene_id)
            .ok_or_else(|| format!("Scene not found: {}", scene_id.as_str()))?
            .clone();

        // Check requirements
        if !scene.rhai_requirements.is_empty() {
            match self.script_executor.eval_condition(
                &scene.rhai_requirements,
                &self.game_state,
                &EmptyTags,
            ) {
                Ok(true) => {}
                Ok(false) => return Err("Scene requirements not met".to_string()),
                Err(e) => {
                    self.script_errors.push(format!("Requirements error: {}", e));
                    // Continue anyway for testing
                }
            }
        }

        self.active = true;
        self.scene_id = Some(scene_id);
        self.passage_index = 0;
        self.script_errors.clear();

        self.load_passage(&scene, 0)?;
        Ok(())
    }

    /// Make a choice.
    pub fn make_choice(&mut self, choice_index: usize) -> Result<(), String> {
        let scene_id = self.scene_id.clone().ok_or("No scene is playing")?;
        let scene = self.content
            .get_scene(&scene_id)
            .ok_or("Scene not found")?
            .clone();

        let passage = scene.passages
            .get(self.passage_index)
            .ok_or("Invalid passage index")?;

        let choice = passage.choices
            .get(choice_index)
            .ok_or("Invalid choice index")?;

        // Execute choice effects
        if !choice.rhai_effects.is_empty() {
            let rng_seed = self.rng.state();
            match self.script_executor.eval(
                &choice.rhai_effects,
                &self.game_state,
                &EmptyTags,
                rng_seed,
            ) {
                Ok(result) => {
                    // Update RNG state
                    self.rng.set_state(result.rng_state);
                    // Apply effects to game state
                    self.apply_effects(&result.effects);
                }
                Err(e) => {
                    self.script_errors.push(format!("Effect error: {}", e));
                }
            }
        }

        // Navigate
        match &choice.next {
            engine_core::Navigation::Passage(idx) => {
                self.passage_index = *idx;
                self.load_passage(&scene, *idx)?;
            }
            engine_core::Navigation::End => {
                self.passage_text = "[Scene End]".to_string();
                self.choices.clear();
            }
        }

        Ok(())
    }

    /// Stop playing.
    pub fn stop(&mut self) {
        self.active = false;
        self.scene_id = None;
        self.passage_text.clear();
        self.choices.clear();
    }

    /// Reset game state (keep scenes).
    pub fn reset_state(&mut self, seed: u64) {
        self.game_state = GameState::default();
        self.rng = Rng::new(seed);
        self.chronicle.clear();
        self.script_errors.clear();
    }

    /// Load a passage, evaluating scripts.
    fn load_passage(&mut self, scene: &Scene, index: usize) -> Result<(), String> {
        let passage = scene.passages
            .get(index)
            .ok_or("Invalid passage index")?;

        // Execute on_enter script
        if let Some(ref script) = passage.rhai_on_enter {
            if !script.is_empty() {
                let rng_seed = self.rng.state();
                match self.script_executor.eval(
                    script,
                    &self.game_state,
                    &EmptyTags,
                    rng_seed,
                ) {
                    Ok(result) => {
                        self.rng.set_state(result.rng_state);
                        self.apply_effects(&result.effects);
                    }
                    Err(e) => {
                        self.script_errors.push(format!("on_enter error: {}", e));
                    }
                }
            }
        }

        // Interpolate passage text (TODO: implement ${expr} interpolation)
        self.passage_text = passage.text.to_string();

        // Evaluate choice conditions
        self.choices = passage.choices
            .iter()
            .enumerate()
            .map(|(i, choice)| {
                let (enabled, reason) = if choice.rhai_condition.is_empty() {
                    (true, None)
                } else {
                    match self.script_executor.eval_condition(
                        &choice.rhai_condition,
                        &self.game_state,
                        &EmptyTags,
                    ) {
                        Ok(true) => (true, None),
                        Ok(false) => (false, Some("Condition not met".to_string())),
                        Err(e) => {
                            self.script_errors.push(format!("Condition error: {}", e));
                            (false, Some(format!("Script error: {}", e)))
                        }
                    }
                };

                FullPlayerChoice {
                    index: i,
                    text: choice.text.to_string(),
                    enabled,
                    disabled_reason: reason,
                }
            })
            .collect();

        Ok(())
    }

    /// Apply a list of effects to the game state.
    fn apply_effects(&mut self, effects: &[Effect]) {
        for effect in effects {
            match effect {
                Effect::ModifyResource { resource, delta } => {
                    self.game_state.modify_resource(resource, *delta);
                }
                Effect::SetResource { resource, value } => {
                    self.game_state.set_resource(resource.clone(), *value);
                }
                Effect::SetFlag { flag, value } => {
                    self.game_state.set_flag(flag.clone(), value.clone());
                }
                Effect::AddChronicle { title, text } => {
                    self.chronicle.push(ChronicleEntry {
                        title: title.to_string(),
                        text: text.to_string(),
                    });
                }
                Effect::ModifyFactionReputation { faction, delta } => {
                    self.game_state.modify_faction_reputation(faction, *delta);
                }
                Effect::Damage { resource, amount } => {
                    // Damage is negative modification
                    self.game_state.modify_resource(resource, -*amount);
                }
                Effect::ModifyStat { key, delta } => {
                    self.game_state.increment_stat(key.clone(), *delta);
                }
                // Card/module effects - not fully supported in holdsmith preview
                Effect::AddCard { .. } |
                Effect::RemoveCards { .. } |
                Effect::EquipModule { .. } |
                Effect::UnequipModule { .. } => {
                    // Skip card-related effects for now
                }
                Effect::Script { source } => {
                    // Nested script execution
                    let rng_seed = self.rng.state();
                    if let Ok(result) = self.script_executor.eval(
                        source,
                        &self.game_state,
                        &EmptyTags,
                        rng_seed,
                    ) {
                        self.rng.set_state(result.rng_state);
                        // Recursive apply (careful with deep nesting)
                        self.apply_effects(&result.effects);
                    }
                }
                Effect::Compound { effects } => {
                    self.apply_effects(effects);
                }
            }
        }
    }

    /// Set a resource value (for testing).
    pub fn set_resource(&mut self, name: &str, value: i64) {
        self.game_state.set_resource(ResourceId::new(name), value);
    }

    /// Set a flag value (for testing).
    pub fn set_flag(&mut self, name: &str, value: Value) {
        self.game_state.set_flag(FlagId::new(name), value);
    }
}

/// A choice in the full player.
#[derive(Debug, Clone)]
pub struct FullPlayerChoice {
    pub index: usize,
    pub text: String,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
}

/// A chronicle entry.
#[derive(Debug, Clone)]
pub struct ChronicleEntry {
    pub title: String,
    pub text: String,
}

/// Empty tag provider for script evaluation.
struct EmptyTags;

impl TagProvider for EmptyTags {
    fn has_tag(&self, _category: &TagCategoryId, _tag: &TagId) -> bool {
        false
    }
}

// ============================================================================
// Snapshot for UI
// ============================================================================

/// Serializable snapshot of full player state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FullPlayerSnapshot {
    pub active: bool,
    pub scene_id: Option<String>,
    pub passage_index: usize,
    pub passage_text: String,
    pub choices: Vec<FullPlayerChoiceSnapshot>,
    pub resources: Vec<ResourceSnapshot>,
    pub flags: Vec<FlagSnapshot>,
    pub chronicle: Vec<ChronicleSnapshot>,
    pub script_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullPlayerChoiceSnapshot {
    pub index: usize,
    pub text: String,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub name: String,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagSnapshot {
    pub name: String,
    pub value: String, // Serialized as string for simplicity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleSnapshot {
    pub title: String,
    pub text: String,
}

impl FullPlayerState {
    /// Create a serializable snapshot.
    pub fn snapshot(&self) -> FullPlayerSnapshot {
        FullPlayerSnapshot {
            active: self.active,
            scene_id: self.scene_id.as_ref().map(|id| id.as_str().to_string()),
            passage_index: self.passage_index,
            passage_text: self.passage_text.clone(),
            choices: self.choices.iter().map(|c| FullPlayerChoiceSnapshot {
                index: c.index,
                text: c.text.clone(),
                enabled: c.enabled,
                disabled_reason: c.disabled_reason.clone(),
            }).collect(),
            resources: self.game_state.resources.iter().map(|(k, v)| ResourceSnapshot {
                name: k.as_str().to_string(),
                value: *v,
            }).collect(),
            flags: self.game_state.flags.iter().map(|(k, v)| FlagSnapshot {
                name: k.as_str().to_string(),
                value: format!("{:?}", v),
            }).collect(),
            chronicle: self.chronicle.iter().map(|e| ChronicleSnapshot {
                title: e.title.clone(),
                text: e.text.clone(),
            }).collect(),
            script_errors: self.script_errors.clone(),
        }
    }
}
