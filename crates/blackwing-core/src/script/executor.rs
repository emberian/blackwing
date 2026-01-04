//! Script executor that runs Rhai scripts with world state access.

use std::sync::Arc;

use rhai::{Dynamic, Engine, ImmutableString, Scope, AST};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

use crate::ui::{UiChoice, UiNode, UiResource};
use crate::{EntityId, FlagId, ResourceId, ScopeKind, Value, WorldState};

use super::shared_state::{ScriptEffect, ScriptState};

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

/// Convert a Value to a Rhai Dynamic (public for use by ScriptSystem)
pub fn value_to_dynamic_public(value: &Value) -> Dynamic {
    value_to_dynamic(value)
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

/// Script executor that handles Rhai script execution.
pub struct ScriptExecutor {
    engine: Engine,
}

impl ScriptExecutor {
    /// Create a new executor with base configuration
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Safety limits
        engine.set_max_expr_depths(64, 32);
        engine.set_max_operations(10_000);
        engine.set_max_string_size(4096);
        engine.set_max_array_size(1000);
        engine.set_max_map_size(100);
        engine.set_strict_variables(false);

        Self { engine }
    }

    /// Compile a script to an AST for repeated execution
    pub fn compile(&self, source: &str) -> Result<AST, rhai::ParseError> {
        self.engine.compile(source)
    }

    /// Execute a script with world state access.
    /// Returns the collected effects.
    pub fn execute(
        &self,
        ast: &AST,
        world: &WorldState,
        script_state: ScriptState,
    ) -> Result<Vec<ScriptEffect>, Box<rhai::EvalAltResult>> {
        let snapshot = Arc::new(WorldSnapshot::from_world(world));
        let engine = self.build_engine_with_bindings(snapshot, script_state.clone());

        let mut scope = Scope::new();
        engine.run_ast_with_scope(&mut scope, ast)?;

        Ok(script_state.into_effects())
    }

    /// Call a specific function in the AST with arguments
    pub fn call_fn<T: Clone + Send + Sync + 'static>(
        &self,
        ast: &AST,
        world: &WorldState,
        script_state: ScriptState,
        fn_name: &str,
        args: impl rhai::FuncArgs,
    ) -> Result<(T, Vec<ScriptEffect>), Box<rhai::EvalAltResult>> {
        let snapshot = Arc::new(WorldSnapshot::from_world(world));
        let engine = self.build_engine_with_bindings(snapshot, script_state.clone());

        let mut scope = Scope::new();
        let result: T = engine.call_fn(&mut scope, ast, fn_name, args)?;

        Ok((result, script_state.into_effects()))
    }

    /// Build an engine with all world bindings registered
    fn build_engine_with_bindings(
        &self,
        snapshot: Arc<WorldSnapshot>,
        state: ScriptState,
    ) -> Engine {
        let mut engine = Engine::new();

        // Copy safety settings
        engine.set_max_expr_depths(64, 32);
        engine.set_max_operations(10_000);
        engine.set_max_string_size(4096);
        engine.set_max_array_size(1000);
        engine.set_max_map_size(100);
        engine.set_strict_variables(false);

        // Register all bindings
        self.register_entity_bindings(&mut engine, snapshot.clone(), state.clone());
        self.register_resource_bindings(&mut engine, snapshot.clone(), state.clone());
        self.register_flag_bindings(&mut engine, snapshot.clone(), state.clone());
        self.register_rng_bindings(&mut engine, state.clone());
        self.register_effect_bindings(&mut engine, state.clone());
        self.register_ui_bindings(&mut engine, state.clone());

        engine
    }

    fn register_entity_bindings(
        &self,
        engine: &mut Engine,
        snapshot: Arc<WorldSnapshot>,
        state: ScriptState,
    ) {
        // entity_exists(qualified_id)
        let snap = snapshot.clone();
        engine.register_fn("entity_exists", move |qualified_id: ImmutableString| -> bool {
            snap.entities.contains_key(qualified_id.as_str())
        });

        // get_component(qualified_id, component)
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

        // entity_has_tag(qualified_id, tag)
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

        // entities_of_kind(kind)
        let snap = snapshot.clone();
        engine.register_fn("entities_of_kind", move |kind: ImmutableString| -> rhai::Array {
            snap.entities
                .values()
                .filter(|e| e.kind == kind.as_str())
                .map(|e| Dynamic::from(format!("{}:{}", e.kind, e.id)))
                .collect()
        });

        // entities_with_tag(tag)
        let snap = snapshot.clone();
        engine.register_fn("entities_with_tag", move |tag: ImmutableString| -> rhai::Array {
            snap.entities
                .values()
                .filter(|e| e.tags.iter().any(|t| t == tag.as_str()))
                .map(|e| Dynamic::from(format!("{}:{}", e.kind, e.id)))
                .collect()
        });

        // spawn_entity(kind, id)
        let st = state.clone();
        engine.register_fn(
            "spawn_entity",
            move |kind: ImmutableString, id: ImmutableString| {
                st.push_effect(ScriptEffect::SpawnEntity {
                    kind: SmolStr::new(kind.as_str()),
                    id: SmolStr::new(id.as_str()),
                });
            },
        );

        // despawn_entity(qualified_id)
        let st = state.clone();
        engine.register_fn("despawn_entity", move |qualified_id: ImmutableString| {
            if let Some(entity_id) = EntityId::parse(qualified_id.as_str()) {
                st.push_effect(ScriptEffect::DespawnEntity { entity_id });
            }
        });

        // set_component(qualified_id, component, value)
        let st = state.clone();
        engine.register_fn(
            "set_component",
            move |qualified_id: ImmutableString, component: ImmutableString, value: Dynamic| {
                if let Some(entity_id) = EntityId::parse(qualified_id.as_str()) {
                    st.push_effect(ScriptEffect::SetComponent {
                        entity_id,
                        component: SmolStr::new(component.as_str()),
                        value: dynamic_to_value(value),
                    });
                }
            },
        );

        // add_tag(qualified_id, tag)
        let st = state.clone();
        engine.register_fn(
            "add_tag",
            move |qualified_id: ImmutableString, tag: ImmutableString| {
                if let Some(entity_id) = EntityId::parse(qualified_id.as_str()) {
                    st.push_effect(ScriptEffect::AddTag {
                        entity_id,
                        tag: SmolStr::new(tag.as_str()),
                    });
                }
            },
        );

        // remove_tag(qualified_id, tag)
        let st = state.clone();
        engine.register_fn(
            "remove_tag",
            move |qualified_id: ImmutableString, tag: ImmutableString| {
                if let Some(entity_id) = EntityId::parse(qualified_id.as_str()) {
                    st.push_effect(ScriptEffect::RemoveTag {
                        entity_id,
                        tag: SmolStr::new(tag.as_str()),
                    });
                }
            },
        );
    }

    fn register_resource_bindings(
        &self,
        engine: &mut Engine,
        snapshot: Arc<WorldSnapshot>,
        state: ScriptState,
    ) {
        // resource(name)
        let snap = snapshot.clone();
        engine.register_fn("resource", move |name: ImmutableString| -> i64 {
            snap.resources.get(name.as_str()).copied().unwrap_or(0)
        });

        // set_resource(name, value)
        let st = state.clone();
        engine.register_fn("set_resource", move |name: ImmutableString, value: i64| {
            st.push_effect(ScriptEffect::SetResource {
                resource: ResourceId::new(name.as_str()),
                value,
            });
        });

        // modify_resource(name, delta)
        let st = state.clone();
        engine.register_fn("modify_resource", move |name: ImmutableString, delta: i64| {
            st.push_effect(ScriptEffect::ModifyResource {
                resource: ResourceId::new(name.as_str()),
                delta,
            });
        });
    }

    fn register_flag_bindings(
        &self,
        engine: &mut Engine,
        snapshot: Arc<WorldSnapshot>,
        state: ScriptState,
    ) {
        // flag(name)
        let snap = snapshot.clone();
        engine.register_fn("flag", move |name: ImmutableString| -> Dynamic {
            snap.flags.get(name.as_str()).cloned().unwrap_or(Dynamic::UNIT)
        });

        // flag_bool(name)
        let snap = snapshot.clone();
        engine.register_fn("flag_bool", move |name: ImmutableString| -> bool {
            snap.flags
                .get(name.as_str())
                .and_then(|d| d.as_bool().ok())
                .unwrap_or(false)
        });

        // set_flag(name, value) - bool version
        let st = state.clone();
        engine.register_fn("set_flag", move |name: ImmutableString, value: bool| {
            st.push_effect(ScriptEffect::SetFlag {
                flag: FlagId::new(name.as_str()),
                value: Value::Bool(value),
            });
        });

        // set_flag_int(name, value)
        let st = state.clone();
        engine.register_fn("set_flag_int", move |name: ImmutableString, value: i64| {
            st.push_effect(ScriptEffect::SetFlag {
                flag: FlagId::new(name.as_str()),
                value: Value::Int(value),
            });
        });

        // clear_flag(name)
        let st = state.clone();
        engine.register_fn("clear_flag", move |name: ImmutableString| {
            st.push_effect(ScriptEffect::ClearFlag {
                flag: FlagId::new(name.as_str()),
            });
        });
    }

    fn register_rng_bindings(&self, engine: &mut Engine, state: ScriptState) {
        // rng_float() -> f64 in [0, 1)
        let rng = state.rng_state();
        engine.register_fn("rng_float", move || -> f64 {
            let mut st = rng.lock().unwrap();
            *st = st
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (*st >> 11) as f64 / (1u64 << 53) as f64
        });

        // rng_int(min, max) -> i64 in [min, max)
        let rng = state.rng_state();
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

        // rng_chance(probability) -> bool
        let rng = state.rng_state();
        engine.register_fn("rng_chance", move |probability: f64| -> bool {
            let mut st = rng.lock().unwrap();
            *st = st
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let f = (*st >> 11) as f64 / (1u64 << 53) as f64;
            f < probability
        });
    }

    fn register_effect_bindings(&self, engine: &mut Engine, state: ScriptState) {
        // chronicle(title, description)
        let st = state.clone();
        engine.register_fn(
            "chronicle",
            move |title: ImmutableString, description: ImmutableString| {
                st.push_effect(ScriptEffect::Chronicle {
                    title: SmolStr::new(title.as_str()),
                    description: SmolStr::new(description.as_str()),
                });
            },
        );

        // advance_time(ticks)
        let st = state.clone();
        engine.register_fn("advance_time", move |ticks: i64| {
            if ticks > 0 {
                st.push_effect(ScriptEffect::AdvanceTime {
                    ticks: ticks as u64,
                });
            }
        });

        // push_scope(kind, id) - uses current actor
        let st = state.clone();
        engine.register_fn(
            "push_scope",
            move |kind: ImmutableString, id: ImmutableString| {
                if let Some(actor) = st.actor() {
                    st.push_effect(ScriptEffect::PushScope {
                        actor,
                        kind: ScopeKind::from_str(kind.as_str()),
                        id: SmolStr::new(id.as_str()),
                    });
                }
            },
        );

        // pop_scope() - uses current actor
        let st = state.clone();
        engine.register_fn("pop_scope", move || {
            if let Some(actor) = st.actor() {
                st.push_effect(ScriptEffect::PopScope { actor });
            }
        });
    }

    fn register_ui_bindings(&self, engine: &mut Engine, state: ScriptState) {
        // === Layout composition functions ===

        // vstack(children) - vertical stack
        engine.register_fn("vstack", |children: rhai::Array| -> Dynamic {
            let nodes = dynamic_array_to_ui_nodes(children);
            ui_node_to_dynamic(UiNode::VStack { children: nodes, gap: None })
        });

        // vstack(children, gap) - vertical stack with gap
        engine.register_fn("vstack", |children: rhai::Array, gap: f64| -> Dynamic {
            let nodes = dynamic_array_to_ui_nodes(children);
            ui_node_to_dynamic(UiNode::VStack {
                children: nodes,
                gap: Some(gap as f32),
            })
        });

        // hstack(children) - horizontal stack
        engine.register_fn("hstack", |children: rhai::Array| -> Dynamic {
            let nodes = dynamic_array_to_ui_nodes(children);
            ui_node_to_dynamic(UiNode::HStack { children: nodes, gap: None })
        });

        // hstack(children, gap) - horizontal stack with gap
        engine.register_fn("hstack", |children: rhai::Array, gap: f64| -> Dynamic {
            let nodes = dynamic_array_to_ui_nodes(children);
            ui_node_to_dynamic(UiNode::HStack {
                children: nodes,
                gap: Some(gap as f32),
            })
        });

        // card(title, children) - card container
        engine.register_fn(
            "card",
            |title: ImmutableString, children: rhai::Array| -> Dynamic {
                let nodes = dynamic_array_to_ui_nodes(children);
                let title = if title.is_empty() {
                    None
                } else {
                    Some(SmolStr::new(title.as_str()))
                };
                ui_node_to_dynamic(UiNode::Card {
                    title,
                    children: nodes,
                })
            },
        );

        // card_no_title(children) - card without title
        engine.register_fn("card_no_title", |children: rhai::Array| -> Dynamic {
            let nodes = dynamic_array_to_ui_nodes(children);
            ui_node_to_dynamic(UiNode::Card {
                title: None,
                children: nodes,
            })
        });

        // === Content creation functions ===

        // passage(text) - narrative text
        engine.register_fn("passage", |text: ImmutableString| -> Dynamic {
            ui_node_to_dynamic(UiNode::Passage {
                text: SmolStr::new(text.as_str()),
            })
        });

        // text(text) - simple text
        engine.register_fn("text", |text: ImmutableString| -> Dynamic {
            ui_node_to_dynamic(UiNode::Text {
                text: SmolStr::new(text.as_str()),
            })
        });

        // choices(items) - choice list from array of maps
        engine.register_fn("choices", |items: rhai::Array| -> Dynamic {
            let choices: Vec<UiChoice> = items
                .into_iter()
                .filter_map(|item| dynamic_to_ui_choice(item))
                .collect();
            ui_node_to_dynamic(UiNode::Choices { items: choices })
        });

        // button(text, action) - clickable button
        engine.register_fn(
            "button",
            |text: ImmutableString, action: ImmutableString| -> Dynamic {
                ui_node_to_dynamic(UiNode::Button {
                    text: SmolStr::new(text.as_str()),
                    action: SmolStr::new(action.as_str()),
                    enabled: true,
                })
            },
        );

        // button(text, action, enabled) - button with enabled state
        engine.register_fn(
            "button",
            |text: ImmutableString, action: ImmutableString, enabled: bool| -> Dynamic {
                ui_node_to_dynamic(UiNode::Button {
                    text: SmolStr::new(text.as_str()),
                    action: SmolStr::new(action.as_str()),
                    enabled,
                })
            },
        );

        // resource_bar(resources) - resource display
        engine.register_fn("resource_bar", |resources: rhai::Array| -> Dynamic {
            let res: Vec<UiResource> = resources
                .into_iter()
                .filter_map(|item| dynamic_to_ui_resource(item))
                .collect();
            ui_node_to_dynamic(UiNode::ResourceBar { resources: res })
        });

        // === Special nodes ===

        // empty() - empty node
        engine.register_fn("empty", || -> Dynamic { ui_node_to_dynamic(UiNode::Empty) });

        // spacer() - flexible spacer
        engine.register_fn("spacer", || -> Dynamic {
            ui_node_to_dynamic(UiNode::Spacer)
        });

        // divider() - horizontal line
        engine.register_fn("divider", || -> Dynamic {
            ui_node_to_dynamic(UiNode::Divider)
        });

        // === Reactive bindings ===

        // bound(path, fallback) - bind to world state
        engine.register_fn(
            "bound",
            |path: ImmutableString, fallback: Dynamic| -> Dynamic {
                let fallback_node = dynamic_to_ui_node(fallback).unwrap_or(UiNode::Empty);
                ui_node_to_dynamic(UiNode::Bound {
                    path: SmolStr::new(path.as_str()),
                    fallback: Box::new(fallback_node),
                })
            },
        );

        // when(condition, then_node) - conditional rendering
        engine.register_fn(
            "when",
            |condition: ImmutableString, then_node: Dynamic| -> Dynamic {
                let then_ui = dynamic_to_ui_node(then_node).unwrap_or(UiNode::Empty);
                ui_node_to_dynamic(UiNode::Conditional {
                    condition: SmolStr::new(condition.as_str()),
                    then_node: Box::new(then_ui),
                    else_node: None,
                })
            },
        );

        // when_else(condition, then_node, else_node) - conditional with else
        engine.register_fn(
            "when_else",
            |condition: ImmutableString, then_node: Dynamic, else_node: Dynamic| -> Dynamic {
                let then_ui = dynamic_to_ui_node(then_node).unwrap_or(UiNode::Empty);
                let else_ui = dynamic_to_ui_node(else_node);
                ui_node_to_dynamic(UiNode::Conditional {
                    condition: SmolStr::new(condition.as_str()),
                    then_node: Box::new(then_ui),
                    else_node: else_ui.map(Box::new),
                })
            },
        );

        // === UI effect functions ===

        // set_ui(root) - set the root UI
        let st = state.clone();
        engine.register_fn("set_ui", move |root: Dynamic| {
            if let Some(node) = dynamic_to_ui_node(root) {
                st.push_effect(ScriptEffect::SetUiRoot { root: node });
            }
        });

        // show_modal(content) - show modal (non-blocking)
        let st = state.clone();
        engine.register_fn("show_modal", move |content: Dynamic| {
            if let Some(node) = dynamic_to_ui_node(content) {
                st.push_effect(ScriptEffect::ShowModal {
                    content: node,
                    blocking: false,
                });
            }
        });

        // show_modal_blocking(content) - show blocking modal
        let st = state.clone();
        engine.register_fn("show_modal_blocking", move |content: Dynamic| {
            if let Some(node) = dynamic_to_ui_node(content) {
                st.push_effect(ScriptEffect::ShowModal {
                    content: node,
                    blocking: true,
                });
            }
        });

        // close_modal() - close current modal
        let st = state.clone();
        engine.register_fn("close_modal", move || {
            st.push_effect(ScriptEffect::CloseModal);
        });

        // clear_ui() - clear the UI
        let st = state.clone();
        engine.register_fn("clear_ui", move || {
            st.push_effect(ScriptEffect::ClearUi);
        });
    }
}

// === Helper functions for UI conversion ===

/// Convert a Dynamic (from Rhai) to a UiNode
fn dynamic_to_ui_node(dyn_val: Dynamic) -> Option<UiNode> {
    // If it's unit/null, return None
    if dyn_val.is_unit() {
        return None;
    }

    // Try to read it as our serialized UiNode format
    if let Some(map) = dyn_val.try_cast::<rhai::Map>() {
        // Check for "type" field to determine node kind
        if let Some(type_val) = map.get("type") {
            if let Some(type_str) = type_val.clone().into_immutable_string().ok() {
                return match type_str.as_str() {
                    "v_stack" | "vstack" => {
                        let children = map
                            .get("children")
                            .and_then(|c: &Dynamic| c.clone().try_cast::<rhai::Array>())
                            .map(dynamic_array_to_ui_nodes)
                            .unwrap_or_default();
                        let gap = map
                            .get("gap")
                            .and_then(|g: &Dynamic| g.as_float().ok())
                            .map(|g| g as f32);
                        Some(UiNode::VStack { children, gap })
                    }
                    "h_stack" | "hstack" => {
                        let children = map
                            .get("children")
                            .and_then(|c: &Dynamic| c.clone().try_cast::<rhai::Array>())
                            .map(dynamic_array_to_ui_nodes)
                            .unwrap_or_default();
                        let gap = map
                            .get("gap")
                            .and_then(|g: &Dynamic| g.as_float().ok())
                            .map(|g| g as f32);
                        Some(UiNode::HStack { children, gap })
                    }
                    "passage" => {
                        let text = map
                            .get("text")
                            .and_then(|t: &Dynamic| t.clone().into_immutable_string().ok())
                            .map(|s| SmolStr::new(s.as_str()))
                            .unwrap_or_default();
                        Some(UiNode::Passage { text })
                    }
                    "text" => {
                        let text = map
                            .get("text")
                            .and_then(|t: &Dynamic| t.clone().into_immutable_string().ok())
                            .map(|s| SmolStr::new(s.as_str()))
                            .unwrap_or_default();
                        Some(UiNode::Text { text })
                    }
                    "choices" => {
                        let items = map
                            .get("items")
                            .and_then(|i: &Dynamic| i.clone().try_cast::<rhai::Array>())
                            .map(|arr| {
                                arr.into_iter()
                                    .filter_map(dynamic_to_ui_choice)
                                    .collect()
                            })
                            .unwrap_or_default();
                        Some(UiNode::Choices { items })
                    }
                    "button" => {
                        let text = map
                            .get("text")
                            .and_then(|t: &Dynamic| t.clone().into_immutable_string().ok())
                            .map(|s| SmolStr::new(s.as_str()))
                            .unwrap_or_default();
                        let action = map
                            .get("action")
                            .and_then(|a: &Dynamic| a.clone().into_immutable_string().ok())
                            .map(|s| SmolStr::new(s.as_str()))
                            .unwrap_or_default();
                        let enabled = map
                            .get("enabled")
                            .and_then(|e: &Dynamic| e.as_bool().ok())
                            .unwrap_or(true);
                        Some(UiNode::Button {
                            text,
                            action,
                            enabled,
                        })
                    }
                    "card" => {
                        let title = map
                            .get("title")
                            .and_then(|t: &Dynamic| t.clone().into_immutable_string().ok())
                            .filter(|s| !s.is_empty())
                            .map(|s| SmolStr::new(s.as_str()));
                        let children = map
                            .get("children")
                            .and_then(|c: &Dynamic| c.clone().try_cast::<rhai::Array>())
                            .map(dynamic_array_to_ui_nodes)
                            .unwrap_or_default();
                        Some(UiNode::Card { title, children })
                    }
                    "empty" => Some(UiNode::Empty),
                    "spacer" => Some(UiNode::Spacer),
                    "divider" => Some(UiNode::Divider),
                    _ => None,
                };
            }
        }
    }

    None
}

/// Convert a UiNode to a Dynamic (for returning from Rhai functions)
fn ui_node_to_dynamic(node: UiNode) -> Dynamic {
    // Serialize to JSON and then to Rhai map
    // This is a simple approach; could be optimized
    match serde_json::to_value(&node) {
        Ok(json) => json_to_dynamic(json),
        Err(_) => Dynamic::UNIT,
    }
}

/// Convert JSON value to Rhai Dynamic
fn json_to_dynamic(value: serde_json::Value) -> Dynamic {
    match value {
        serde_json::Value::Null => Dynamic::UNIT,
        serde_json::Value::Bool(b) => Dynamic::from(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::UNIT
            }
        }
        serde_json::Value::String(s) => Dynamic::from(s),
        serde_json::Value::Array(arr) => {
            let items: Vec<Dynamic> = arr.into_iter().map(json_to_dynamic).collect();
            Dynamic::from(items)
        }
        serde_json::Value::Object(map) => {
            let mut rhai_map = rhai::Map::new();
            for (k, v) in map {
                rhai_map.insert(k.into(), json_to_dynamic(v));
            }
            Dynamic::from(rhai_map)
        }
    }
}

/// Convert an array of Dynamics to Vec<UiNode>
fn dynamic_array_to_ui_nodes(arr: rhai::Array) -> Vec<UiNode> {
    arr.into_iter()
        .filter_map(dynamic_to_ui_node)
        .collect()
}

/// Convert a Dynamic map to UiChoice
fn dynamic_to_ui_choice(dyn_val: Dynamic) -> Option<UiChoice> {
    if let Some(map) = dyn_val.try_cast::<rhai::Map>() {
        let text = map
            .get("text")
            .and_then(|t: &Dynamic| t.clone().into_immutable_string().ok())
            .map(|s| SmolStr::new(s.as_str()))?;
        let action = map
            .get("action")
            .and_then(|a: &Dynamic| a.clone().into_immutable_string().ok())
            .map(|s| SmolStr::new(s.as_str()))
            .unwrap_or_default();
        let enabled = map
            .get("enabled")
            .and_then(|e: &Dynamic| e.as_bool().ok())
            .unwrap_or(true);
        let disabled_reason = map
            .get("disabled_reason")
            .and_then(|r: &Dynamic| r.clone().into_immutable_string().ok())
            .filter(|s| !s.is_empty())
            .map(|s| SmolStr::new(s.as_str()));

        Some(UiChoice {
            text,
            action,
            enabled,
            disabled_reason,
        })
    } else {
        None
    }
}

/// Convert a Dynamic map to UiResource
fn dynamic_to_ui_resource(dyn_val: Dynamic) -> Option<UiResource> {
    if let Some(map) = dyn_val.try_cast::<rhai::Map>() {
        let id = map
            .get("id")
            .and_then(|i: &Dynamic| i.clone().into_immutable_string().ok())
            .map(|s| SmolStr::new(s.as_str()))?;
        let label = map
            .get("label")
            .and_then(|l: &Dynamic| l.clone().into_immutable_string().ok())
            .map(|s| SmolStr::new(s.as_str()))
            .unwrap_or_else(|| id.clone());
        let icon = map
            .get("icon")
            .and_then(|i: &Dynamic| i.clone().into_immutable_string().ok())
            .filter(|s| !s.is_empty())
            .map(|s| SmolStr::new(s.as_str()));
        let current = map
            .get("current")
            .and_then(|c: &Dynamic| c.as_int().ok())
            .unwrap_or(0);
        let max = map.get("max").and_then(|m: &Dynamic| m.as_int().ok());
        let warning_threshold = map
            .get("warning")
            .and_then(|w: &Dynamic| w.as_int().ok());
        let critical_threshold = map
            .get("critical")
            .and_then(|c: &Dynamic| c.as_int().ok());

        Some(UiResource {
            id,
            label,
            icon,
            current,
            max,
            warning_threshold,
            critical_threshold,
        })
    } else {
        None
    }
}

impl Default for ScriptExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_and_execute_simple() {
        let executor = ScriptExecutor::new();
        let world = WorldState::new();
        let state = ScriptState::new(12345);

        let ast = executor.compile("let x = 1 + 2;").unwrap();
        let effects = executor.execute(&ast, &world, state).unwrap();

        assert!(effects.is_empty());
    }

    #[test]
    fn execute_with_effects() {
        let executor = ScriptExecutor::new();
        let world = WorldState::new();
        let state = ScriptState::new(12345);

        let ast = executor
            .compile(
                r#"
                modify_resource("gold", 100);
                set_flag("test_flag", true);
                chronicle("Test", "This is a test");
            "#,
            )
            .unwrap();

        let effects = executor.execute(&ast, &world, state).unwrap();

        assert_eq!(effects.len(), 3);
        assert!(matches!(&effects[0], ScriptEffect::ModifyResource { delta, .. } if *delta == 100));
        assert!(matches!(&effects[1], ScriptEffect::SetFlag { .. }));
        assert!(matches!(&effects[2], ScriptEffect::Chronicle { .. }));
    }

    #[test]
    fn rng_is_deterministic() {
        let executor = ScriptExecutor::new();
        let world = WorldState::new();

        let ast = executor.compile("rng_int(0, 100)").unwrap();

        let state1 = ScriptState::new(12345);
        let state2 = ScriptState::new(12345);

        // Same seed should produce same state progression
        let _ = executor.execute(&ast, &world, state1.clone());
        let _ = executor.execute(&ast, &world, state2.clone());

        assert_eq!(state1.final_rng_state(), state2.final_rng_state());
    }
}
