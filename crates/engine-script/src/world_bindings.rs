//! Rhai bindings for the generalized world state.
//!
//! Provides scripting access to:
//! - Entity operations (spawn, query, components, tags)
//! - World state (resources, flags)
//! - Scope management
//!
//! # Example
//!
//! ```rhai
//! // Entity operations
//! let npc = world.spawn("npc", "merchant_bob");
//! world.set_component(npc, "name", "Bob the Merchant");
//! world.set_component(npc, "health", 100);
//! world.add_tag(npc, "friendly");
//!
//! // Queries
//! let enemies = world.query_kind("npc").with_tag("hostile");
//!
//! // Globals
//! world.set_resource("gold", 500);
//! let gold = world.resource("gold");
//!
//! // Flags
//! world.set_flag("visited_town", true);
//! let visited = world.flag("visited_town");
//! ```

use std::sync::{Arc, Mutex};

use engine_primitives::Value;
use engine_world::WorldState;
use rhai::{Dynamic, Engine, ImmutableString};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

/// Effects that can be collected during script execution.
#[derive(Debug, Clone)]
pub enum WorldEffect {
    // Entity effects
    SpawnEntity {
        kind: SmolStr,
        id: SmolStr,
    },
    DespawnEntity {
        kind: SmolStr,
        id: SmolStr,
    },
    SetComponent {
        kind: SmolStr,
        id: SmolStr,
        component: SmolStr,
        value: Value,
    },
    RemoveComponent {
        kind: SmolStr,
        id: SmolStr,
        component: SmolStr,
    },
    AddTag {
        kind: SmolStr,
        id: SmolStr,
        tag: SmolStr,
    },
    RemoveTag {
        kind: SmolStr,
        id: SmolStr,
        tag: SmolStr,
    },

    // Resource effects
    SetResource {
        resource: SmolStr,
        value: i64,
    },
    ModifyResource {
        resource: SmolStr,
        delta: i64,
    },

    // Flag effects
    SetFlag {
        flag: SmolStr,
        value: Value,
    },
    ClearFlag {
        flag: SmolStr,
    },

    // Scope effects
    PushScope {
        kind: SmolStr,
        id: SmolStr,
    },
    PopScope,

    // Chronicle
    Chronicle {
        title: SmolStr,
        description: SmolStr,
    },
}

/// Shared state for collecting effects during script execution.
#[derive(Clone)]
pub struct WorldScriptState {
    effects: Arc<Mutex<Vec<WorldEffect>>>,
    rng_state: Arc<Mutex<u64>>,
}

impl WorldScriptState {
    pub fn new(rng_seed: u64) -> Self {
        Self {
            effects: Arc::new(Mutex::new(Vec::new())),
            rng_state: Arc::new(Mutex::new(rng_seed)),
        }
    }

    pub fn into_effects(self) -> Vec<WorldEffect> {
        Arc::try_unwrap(self.effects)
            .map(|m| m.into_inner().unwrap())
            .unwrap_or_else(|arc| arc.lock().unwrap().clone())
    }

    pub fn rng_state(&self) -> u64 {
        *self.rng_state.lock().unwrap()
    }
}

/// Snapshot of world state for read-only access in scripts.
pub struct WorldSnapshot {
    pub entities: FxHashMap<String, EntitySnapshot>,
    pub resources: FxHashMap<String, i64>,
    pub flags: FxHashMap<String, Dynamic>,
}

/// Snapshot of an entity for script access.
#[derive(Clone)]
pub struct EntitySnapshot {
    pub kind: SmolStr,
    pub id: SmolStr,
    pub components: FxHashMap<SmolStr, Dynamic>,
    pub tags: Vec<SmolStr>,
}

impl WorldSnapshot {
    /// Create a snapshot from world state
    pub fn from_world(state: &WorldState) -> Self {
        let mut entities = FxHashMap::default();
        let mut resources = FxHashMap::default();
        let mut flags = FxHashMap::default();

        // Snapshot entities
        for (key, entity) in state.entities.iter() {
            let qualified_id = entity.id.as_qualified();
            let mut components = FxHashMap::default();

            // Get components for this entity
            if let Some(comp_map) = state.entities.components(key) {
                for (name, value) in comp_map {
                    components.insert(name.clone(), value_to_dynamic(value));
                }
            }

            let snapshot = EntitySnapshot {
                kind: entity.id.kind.clone(),
                id: entity.id.id.clone(),
                components,
                tags: entity.tags.iter().map(|t| SmolStr::new(t.as_str())).collect(),
            };
            entities.insert(qualified_id, snapshot);
        }

        // Snapshot resources
        for (id, &value) in &state.resources {
            resources.insert(id.as_str().to_string(), value);
        }

        // Snapshot flags
        for (id, value) in &state.flags {
            flags.insert(id.as_str().to_string(), value_to_dynamic(value));
        }

        Self {
            entities,
            resources,
            flags,
        }
    }
}

fn value_to_dynamic(value: &Value) -> Dynamic {
    match value {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from(*b),
        Value::Int(n) => Dynamic::from(*n),
        Value::Float(f) => Dynamic::from(*f),
        Value::String(s) => Dynamic::from(s.to_string()),
        Value::Array(arr) => {
            let items: Vec<Dynamic> = arr.iter().map(value_to_dynamic).collect();
            Dynamic::from(items)
        }
        Value::Map(map) => {
            let mut rhai_map = rhai::Map::new();
            for (k, v) in map {
                rhai_map.insert(k.clone().into(), value_to_dynamic(v));
            }
            Dynamic::from(rhai_map)
        }
        Value::EntityRef(id) => Dynamic::from(id.as_qualified()),
    }
}

fn dynamic_to_value(dyn_val: Dynamic) -> Value {
    if dyn_val.is_unit() {
        Value::Null
    } else if let Some(b) = dyn_val.as_bool().ok() {
        Value::Bool(b)
    } else if let Some(n) = dyn_val.as_int().ok() {
        Value::Int(n)
    } else if let Some(f) = dyn_val.as_float().ok() {
        Value::Float(f)
    } else if let Some(s) = dyn_val.into_immutable_string().ok() {
        Value::String(SmolStr::new(s.as_str()))
    } else {
        Value::Null
    }
}

/// Register world bindings with a Rhai engine.
///
/// This adds functions for entity operations, resources, flags, etc.
pub fn register_world_bindings(
    engine: &mut Engine,
    snapshot: Arc<WorldSnapshot>,
    state: WorldScriptState,
) {
    // === Entity Query Functions ===

    // Get entity by qualified ID (e.g., "npc:bob")
    let snap = snapshot.clone();
    engine.register_fn("entity", move |qualified_id: ImmutableString| -> Dynamic {
        snap.entities
            .get(qualified_id.as_str())
            .map(|e| {
                let mut map = rhai::Map::new();
                map.insert("kind".into(), Dynamic::from(e.kind.to_string()));
                map.insert("id".into(), Dynamic::from(e.id.to_string()));
                map.insert("qualified".into(), Dynamic::from(format!("{}:{}", e.kind, e.id)));
                Dynamic::from(map)
            })
            .unwrap_or(Dynamic::UNIT)
    });

    // Check if entity exists
    let snap = snapshot.clone();
    engine.register_fn("entity_exists", move |qualified_id: ImmutableString| -> bool {
        snap.entities.contains_key(qualified_id.as_str())
    });

    // Get entity component
    let snap = snapshot.clone();
    engine.register_fn(
        "get_component",
        move |qualified_id: ImmutableString, component: ImmutableString| -> Dynamic {
            snap.entities
                .get(qualified_id.as_str())
                .and_then(|e| e.components.get(component.as_str()))
                .cloned()
                .unwrap_or(Dynamic::UNIT)
        },
    );

    // Check if entity has tag
    let snap = snapshot.clone();
    engine.register_fn(
        "entity_has_tag",
        move |qualified_id: ImmutableString, tag: ImmutableString| -> bool {
            snap.entities
                .get(qualified_id.as_str())
                .map(|e| e.tags.iter().any(|t| t == tag.as_str()))
                .unwrap_or(false)
        },
    );

    // Query entities by kind
    let snap = snapshot.clone();
    engine.register_fn("entities_of_kind", move |kind: ImmutableString| -> rhai::Array {
        snap.entities
            .values()
            .filter(|e| e.kind == kind.as_str())
            .map(|e| Dynamic::from(format!("{}:{}", e.kind, e.id)))
            .collect()
    });

    // Query entities with tag
    let snap = snapshot.clone();
    engine.register_fn("entities_with_tag", move |tag: ImmutableString| -> rhai::Array {
        snap.entities
            .values()
            .filter(|e| e.tags.iter().any(|t| t == tag.as_str()))
            .map(|e| Dynamic::from(format!("{}:{}", e.kind, e.id)))
            .collect()
    });

    // === Resource Functions ===

    let snap = snapshot.clone();
    engine.register_fn("resource", move |name: ImmutableString| -> i64 {
        snap.resources.get(name.as_str()).copied().unwrap_or(0)
    });

    let st = state.clone();
    engine.register_fn("set_resource", move |name: ImmutableString, value: i64| {
        st.effects.lock().unwrap().push(WorldEffect::SetResource {
            resource: SmolStr::new(name.as_str()),
            value,
        });
    });

    let st = state.clone();
    engine.register_fn("modify_resource", move |name: ImmutableString, delta: i64| {
        st.effects.lock().unwrap().push(WorldEffect::ModifyResource {
            resource: SmolStr::new(name.as_str()),
            delta,
        });
    });

    // === Flag Functions ===

    let snap = snapshot.clone();
    engine.register_fn("flag", move |name: ImmutableString| -> Dynamic {
        snap.flags.get(name.as_str()).cloned().unwrap_or(Dynamic::UNIT)
    });

    let snap = snapshot.clone();
    engine.register_fn("flag_bool", move |name: ImmutableString| -> bool {
        snap.flags
            .get(name.as_str())
            .and_then(|d| d.as_bool().ok())
            .unwrap_or(false)
    });

    let st = state.clone();
    engine.register_fn("set_flag", move |name: ImmutableString, value: bool| {
        st.effects.lock().unwrap().push(WorldEffect::SetFlag {
            flag: SmolStr::new(name.as_str()),
            value: Value::Bool(value),
        });
    });

    let st = state.clone();
    engine.register_fn("set_flag_int", move |name: ImmutableString, value: i64| {
        st.effects.lock().unwrap().push(WorldEffect::SetFlag {
            flag: SmolStr::new(name.as_str()),
            value: Value::Int(value),
        });
    });

    let st = state.clone();
    engine.register_fn(
        "set_flag_str",
        move |name: ImmutableString, value: ImmutableString| {
            st.effects.lock().unwrap().push(WorldEffect::SetFlag {
                flag: SmolStr::new(name.as_str()),
                value: Value::String(SmolStr::new(value.as_str())),
            });
        },
    );

    let st = state.clone();
    engine.register_fn("clear_flag", move |name: ImmutableString| {
        st.effects.lock().unwrap().push(WorldEffect::ClearFlag {
            flag: SmolStr::new(name.as_str()),
        });
    });

    // === Entity Mutation Functions ===

    let st = state.clone();
    engine.register_fn(
        "spawn_entity",
        move |kind: ImmutableString, id: ImmutableString| {
            st.effects.lock().unwrap().push(WorldEffect::SpawnEntity {
                kind: SmolStr::new(kind.as_str()),
                id: SmolStr::new(id.as_str()),
            });
        },
    );

    let st = state.clone();
    engine.register_fn(
        "despawn_entity",
        move |qualified_id: ImmutableString| {
            if let Some((kind, id)) = qualified_id.split_once(':') {
                st.effects.lock().unwrap().push(WorldEffect::DespawnEntity {
                    kind: SmolStr::new(kind),
                    id: SmolStr::new(id),
                });
            }
        },
    );

    let st = state.clone();
    engine.register_fn(
        "set_component",
        move |qualified_id: ImmutableString, component: ImmutableString, value: Dynamic| {
            if let Some((kind, id)) = qualified_id.split_once(':') {
                st.effects.lock().unwrap().push(WorldEffect::SetComponent {
                    kind: SmolStr::new(kind),
                    id: SmolStr::new(id),
                    component: SmolStr::new(component.as_str()),
                    value: dynamic_to_value(value),
                });
            }
        },
    );

    let st = state.clone();
    engine.register_fn(
        "remove_component",
        move |qualified_id: ImmutableString, component: ImmutableString| {
            if let Some((kind, id)) = qualified_id.split_once(':') {
                st.effects.lock().unwrap().push(WorldEffect::RemoveComponent {
                    kind: SmolStr::new(kind),
                    id: SmolStr::new(id),
                    component: SmolStr::new(component.as_str()),
                });
            }
        },
    );

    let st = state.clone();
    engine.register_fn(
        "add_tag",
        move |qualified_id: ImmutableString, tag: ImmutableString| {
            if let Some((kind, id)) = qualified_id.split_once(':') {
                st.effects.lock().unwrap().push(WorldEffect::AddTag {
                    kind: SmolStr::new(kind),
                    id: SmolStr::new(id),
                    tag: SmolStr::new(tag.as_str()),
                });
            }
        },
    );

    let st = state.clone();
    engine.register_fn(
        "remove_tag",
        move |qualified_id: ImmutableString, tag: ImmutableString| {
            if let Some((kind, id)) = qualified_id.split_once(':') {
                st.effects.lock().unwrap().push(WorldEffect::RemoveTag {
                    kind: SmolStr::new(kind),
                    id: SmolStr::new(id),
                    tag: SmolStr::new(tag.as_str()),
                });
            }
        },
    );

    // === Scope Functions ===

    let st = state.clone();
    engine.register_fn(
        "push_scope",
        move |kind: ImmutableString, id: ImmutableString| {
            st.effects.lock().unwrap().push(WorldEffect::PushScope {
                kind: SmolStr::new(kind.as_str()),
                id: SmolStr::new(id.as_str()),
            });
        },
    );

    let st = state.clone();
    engine.register_fn("pop_scope", move || {
        st.effects.lock().unwrap().push(WorldEffect::PopScope);
    });

    // === Chronicle ===

    let st = state.clone();
    engine.register_fn(
        "chronicle",
        move |title: ImmutableString, description: ImmutableString| {
            st.effects.lock().unwrap().push(WorldEffect::Chronicle {
                title: SmolStr::new(title.as_str()),
                description: SmolStr::new(description.as_str()),
            });
        },
    );

    // === RNG Functions ===

    let rng = state.rng_state.clone();
    engine.register_fn("rng_float", move || -> f64 {
        let mut st = rng.lock().unwrap();
        *st = st
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*st >> 11) as f64 / (1u64 << 53) as f64
    });

    let rng = state.rng_state.clone();
    engine.register_fn("rng_int", move |min: i64, max: i64| -> i64 {
        if min >= max {
            return min;
        }
        let mut st = rng.lock().unwrap();
        *st = st
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let f = (*st >> 11) as f64 / (1u64 << 53) as f64;
        min + ((max - min) as f64 * f) as i64
    });

    let rng = state.rng_state.clone();
    engine.register_fn("rng_chance", move |probability: f64| -> bool {
        let mut st = rng.lock().unwrap();
        *st = st
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let f = (*st >> 11) as f64 / (1u64 << 53) as f64;
        f < probability
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_primitives::EntityId;

    fn create_test_world() -> WorldState {
        let mut state = WorldState::new();

        // Add some entities
        let bob = state.entities.spawn(EntityId::new("npc", "bob"));
        state.entities.set_component(bob, "name", Value::String("Bob".into()));
        state.entities.set_component(bob, "health", Value::Int(100));
        state.entities.add_tag(bob, "friendly");
        state.entities.add_tag(bob, "merchant");

        let sword = state.entities.spawn(EntityId::new("item", "sword"));
        state.entities.set_component(sword, "damage", Value::Int(10));
        state.entities.add_tag(sword, "weapon");

        // Add resources
        state.resources.insert("gold".into(), 500);
        state.resources.insert("reputation".into(), 50);

        // Add flags
        state.flags.insert("visited_town".into(), Value::Bool(true));
        state.flags.insert("quest_stage".into(), Value::Int(2));

        state
    }

    #[test]
    fn query_entity() {
        let world = create_test_world();
        let snapshot = Arc::new(WorldSnapshot::from_world(&world));
        let script_state = WorldScriptState::new(12345);

        let mut engine = Engine::new();
        register_world_bindings(&mut engine, snapshot, script_state);

        let result: bool = engine.eval(r#"entity_exists("npc:bob")"#).unwrap();
        assert!(result);

        let result: bool = engine.eval(r#"entity_exists("npc:alice")"#).unwrap();
        assert!(!result);
    }

    #[test]
    fn get_component() {
        let world = create_test_world();
        let snapshot = Arc::new(WorldSnapshot::from_world(&world));
        let script_state = WorldScriptState::new(12345);

        let mut engine = Engine::new();
        register_world_bindings(&mut engine, snapshot, script_state);

        let result: i64 = engine.eval(r#"get_component("npc:bob", "health")"#).unwrap();
        assert_eq!(result, 100);

        let result: String = engine.eval(r#"get_component("npc:bob", "name")"#).unwrap();
        assert_eq!(result, "Bob");
    }

    #[test]
    fn entity_tags() {
        let world = create_test_world();
        let snapshot = Arc::new(WorldSnapshot::from_world(&world));
        let script_state = WorldScriptState::new(12345);

        let mut engine = Engine::new();
        register_world_bindings(&mut engine, snapshot, script_state);

        let result: bool = engine.eval(r#"entity_has_tag("npc:bob", "friendly")"#).unwrap();
        assert!(result);

        let result: bool = engine.eval(r#"entity_has_tag("npc:bob", "hostile")"#).unwrap();
        assert!(!result);
    }

    #[test]
    fn query_by_kind() {
        let world = create_test_world();
        let snapshot = Arc::new(WorldSnapshot::from_world(&world));
        let script_state = WorldScriptState::new(12345);

        let mut engine = Engine::new();
        register_world_bindings(&mut engine, snapshot, script_state);

        let result: rhai::Array = engine.eval(r#"entities_of_kind("npc")"#).unwrap();
        assert_eq!(result.len(), 1);

        let result: rhai::Array = engine.eval(r#"entities_of_kind("item")"#).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn resources() {
        let world = create_test_world();
        let snapshot = Arc::new(WorldSnapshot::from_world(&world));
        let script_state = WorldScriptState::new(12345);

        let mut engine = Engine::new();
        register_world_bindings(&mut engine, snapshot, script_state);

        let result: i64 = engine.eval(r#"resource("gold")"#).unwrap();
        assert_eq!(result, 500);

        let result: i64 = engine.eval(r#"resource("nonexistent")"#).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn flags() {
        let world = create_test_world();
        let snapshot = Arc::new(WorldSnapshot::from_world(&world));
        let script_state = WorldScriptState::new(12345);

        let mut engine = Engine::new();
        register_world_bindings(&mut engine, snapshot, script_state);

        let result: bool = engine.eval(r#"flag_bool("visited_town")"#).unwrap();
        assert!(result);

        let result: i64 = engine.eval(r#"flag("quest_stage")"#).unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn collect_effects() {
        let world = create_test_world();
        let snapshot = Arc::new(WorldSnapshot::from_world(&world));
        let script_state = WorldScriptState::new(12345);

        let mut engine = Engine::new();
        register_world_bindings(&mut engine, snapshot, script_state.clone());

        engine.run(r#"
            spawn_entity("npc", "alice");
            set_component("npc:alice", "health", 50);
            add_tag("npc:alice", "friendly");
            modify_resource("gold", -100);
            set_flag("alice_spawned", true);
            chronicle("NPC Spawned", "Alice has joined!");
        "#).unwrap();

        let effects = script_state.into_effects();
        assert_eq!(effects.len(), 6);

        assert!(matches!(&effects[0], WorldEffect::SpawnEntity { kind, id } if kind == "npc" && id == "alice"));
        assert!(matches!(&effects[1], WorldEffect::SetComponent { component, .. } if component == "health"));
        assert!(matches!(&effects[2], WorldEffect::AddTag { tag, .. } if tag == "friendly"));
        assert!(matches!(&effects[3], WorldEffect::ModifyResource { delta, .. } if *delta == -100));
        assert!(matches!(&effects[4], WorldEffect::SetFlag { .. }));
        assert!(matches!(&effects[5], WorldEffect::Chronicle { .. }));
    }
}
