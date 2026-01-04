//! Entity templates define reusable entity blueprints.
//!
//! Templates are used to spawn entities with predefined components, tags, and
//! behavior scripts.

use blackwing_core::{EntityId, Value};
use indexmap::IndexMap;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A template for spawning entities.
///
/// Contains the default components, tags, and scripts that should be applied
/// when an entity is spawned from this template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTemplate {
    /// The template ID (e.g., "npc:merchant_bob")
    pub id: EntityId,

    /// Default component values
    pub components: IndexMap<SmolStr, Value>,

    /// Tags to apply on spawn
    pub tags: FxHashSet<SmolStr>,

    /// Rhai script to run when entity is spawned
    pub on_spawn: Option<SmolStr>,

    /// Rhai script to run each tick (for NPCs with behavior)
    pub on_tick: Option<SmolStr>,

    /// Rhai script to run when entity is interacted with
    pub on_interact: Option<SmolStr>,
}

impl EntityTemplate {
    /// Create a new empty template with the given ID
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            components: IndexMap::new(),
            tags: FxHashSet::default(),
            on_spawn: None,
            on_tick: None,
            on_interact: None,
        }
    }

    /// Add a component to the template
    pub fn with_component(mut self, name: impl Into<SmolStr>, value: Value) -> Self {
        self.components.insert(name.into(), value);
        self
    }

    /// Add a tag to the template
    pub fn with_tag(mut self, tag: impl Into<SmolStr>) -> Self {
        self.tags.insert(tag.into());
        self
    }

    /// Set the on_spawn script
    pub fn with_on_spawn(mut self, script: impl Into<SmolStr>) -> Self {
        self.on_spawn = Some(script.into());
        self
    }

    /// Set the on_tick script
    pub fn with_on_tick(mut self, script: impl Into<SmolStr>) -> Self {
        self.on_tick = Some(script.into());
        self
    }

    /// Set the on_interact script
    pub fn with_on_interact(mut self, script: impl Into<SmolStr>) -> Self {
        self.on_interact = Some(script.into());
        self
    }

    /// Get the entity kind (e.g., "npc", "item", "location")
    pub fn kind(&self) -> &str {
        &self.id.kind
    }

    /// Get a component value
    pub fn get_component(&self, name: &str) -> Option<&Value> {
        self.components.get(name)
    }

    /// Check if template has a tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_builder() {
        let template = EntityTemplate::new(EntityId::new("npc", "merchant_bob"))
            .with_component("name", Value::String("Bob the Merchant".into()))
            .with_component("health", Value::Int(100))
            .with_tag("friendly")
            .with_tag("merchant")
            .with_on_spawn("set_flag('bob_exists', true);");

        assert_eq!(template.kind(), "npc");
        assert_eq!(
            template.get_component("name"),
            Some(&Value::String("Bob the Merchant".into()))
        );
        assert!(template.has_tag("friendly"));
        assert!(template.has_tag("merchant"));
        assert!(template.on_spawn.is_some());
    }
}
