//! Bundle schema defining the structure of the game world.

use engine_primitives::{EntityId, FlagId, ResourceId, Value};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Schema defining the structure of a game's world.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleSchema {
    /// Resource definitions
    #[serde(default)]
    pub resources: Vec<ResourceDef>,

    /// Entity type definitions
    #[serde(default)]
    pub entity_types: Vec<EntityTypeDef>,

    /// Component definitions
    #[serde(default)]
    pub components: Vec<ComponentDef>,

    /// Initial state when starting a new game
    #[serde(default)]
    pub initial_state: InitialStateDef,
}

/// Definition of a resource type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDef {
    /// Resource identifier
    pub id: ResourceId,

    /// Display name
    pub name: String,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,

    /// Minimum value (if bounded)
    #[serde(default)]
    pub min: Option<i64>,

    /// Maximum value (if bounded)
    #[serde(default)]
    pub max: Option<i64>,

    /// Default/starting value
    #[serde(default)]
    pub default: i64,

    /// Whether this resource is visible to the player
    #[serde(default = "default_true")]
    pub visible: bool,
}

fn default_true() -> bool {
    true
}

/// Definition of an entity type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTypeDef {
    /// Type identifier (e.g., "npc", "item", "location")
    pub id: SmolStr,

    /// Display name
    pub name: String,

    /// Components that must be present on this entity type
    #[serde(default)]
    pub required_components: Vec<SmolStr>,

    /// Components that can optionally be present
    #[serde(default)]
    pub optional_components: Vec<SmolStr>,
}

/// Definition of a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDef {
    /// Component identifier (e.g., "health", "name", "position")
    pub id: SmolStr,

    /// Display name
    pub name: String,

    /// Value type
    pub value_type: ValueType,

    /// Default value
    #[serde(default)]
    pub default: Option<Value>,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

/// Type of a component value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    /// Boolean
    Bool,
    /// Integer
    Int,
    /// Floating point
    Float,
    /// String
    String,
    /// Array of values
    Array,
    /// Map of string keys to values
    Map,
    /// Reference to an entity
    EntityRef,
    /// Any type
    Any,
}

/// Definition of initial game state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InitialStateDef {
    /// Starting resource values
    #[serde(default)]
    pub resources: IndexMap<ResourceId, i64>,

    /// Starting flag values
    #[serde(default)]
    pub flags: IndexMap<FlagId, Value>,

    /// Entity templates to spawn at game start
    #[serde(default)]
    pub spawn_entities: Vec<EntityId>,
}

impl ResourceDef {
    /// Create a new resource definition.
    pub fn new(id: impl Into<ResourceId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            min: None,
            max: None,
            default: 0,
            visible: true,
        }
    }

    /// Set the default value.
    pub fn with_default(mut self, default: i64) -> Self {
        self.default = default;
        self
    }

    /// Set min/max bounds.
    pub fn with_bounds(mut self, min: i64, max: i64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }
}

impl EntityTypeDef {
    /// Create a new entity type definition.
    pub fn new(id: impl Into<SmolStr>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            required_components: Vec::new(),
            optional_components: Vec::new(),
        }
    }

    /// Add a required component.
    pub fn with_required(mut self, component: impl Into<SmolStr>) -> Self {
        self.required_components.push(component.into());
        self
    }

    /// Add an optional component.
    pub fn with_optional(mut self, component: impl Into<SmolStr>) -> Self {
        self.optional_components.push(component.into());
        self
    }
}

impl ComponentDef {
    /// Create a new component definition.
    pub fn new(id: impl Into<SmolStr>, name: impl Into<String>, value_type: ValueType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            value_type,
            default: None,
            description: None,
        }
    }

    /// Set the default value.
    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_resource_def() {
        let res = ResourceDef::new("gold", "Gold")
            .with_default(100)
            .with_bounds(0, 99999);

        assert_eq!(res.id.as_str(), "gold");
        assert_eq!(res.default, 100);
        assert_eq!(res.min, Some(0));
    }

    #[test]
    fn create_entity_type_def() {
        let npc = EntityTypeDef::new("npc", "Non-Player Character")
            .with_required("name")
            .with_required("health")
            .with_optional("inventory");

        assert_eq!(npc.required_components.len(), 2);
        assert_eq!(npc.optional_components.len(), 1);
    }

    #[test]
    fn serialize_schema() {
        let mut schema = BundleSchema::default();
        schema.resources.push(ResourceDef::new("gold", "Gold").with_default(100));
        schema.initial_state.resources.insert("gold".into(), 500);

        let toml_str = toml::to_string(&schema).unwrap();
        assert!(toml_str.contains("gold"));
    }
}
