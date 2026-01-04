use std::sync::Arc;

use engine_core::{Effect, GameState, TagCategoryId, TagId, TagProvider, Value};
use parking_lot::Mutex;
use rhai::{Dynamic, Engine, ImmutableString, Scope, AST};
use smol_str::SmolStr;

use crate::error::ScriptError;

/// Result of script execution
#[derive(Debug, Default)]
pub struct ScriptResult {
    /// Effects collected during execution (converted to events by caller)
    pub effects: Vec<Effect>,
    /// Optional navigation instruction from goto()
    pub goto: Option<SmolStr>,
    /// Final RNG state after script execution
    pub rng_state: u64,
}

/// Shared state accumulated during script execution.
/// Uses Arc<Mutex<_>> to be Send+Sync for Rhai's sync feature.
#[derive(Clone)]
struct SharedState {
    effects: Arc<Mutex<Vec<Effect>>>,
    goto: Arc<Mutex<Option<String>>>,
    rng_state: Arc<Mutex<u64>>,
}

impl SharedState {
    fn new(rng_seed: u64) -> Self {
        Self {
            effects: Arc::new(Mutex::new(Vec::new())),
            goto: Arc::new(Mutex::new(None)),
            rng_state: Arc::new(Mutex::new(rng_seed)),
        }
    }

    fn into_result(self) -> ScriptResult {
        ScriptResult {
            effects: Arc::try_unwrap(self.effects)
                .map(|m| m.into_inner())
                .unwrap_or_else(|arc| arc.lock().clone()),
            goto: Arc::try_unwrap(self.goto)
                .map(|m| m.into_inner())
                .unwrap_or_else(|arc| arc.lock().clone())
                .map(SmolStr::new),
            rng_state: Arc::try_unwrap(self.rng_state)
                .map(|m| m.into_inner())
                .unwrap_or_else(|arc| *arc.lock()),
        }
    }
}

/// Script executor that handles the complete execution flow.
///
/// Creates a Rhai engine with game functions registered, executes scripts,
/// and returns collected effects.
pub struct ScriptExecutor {
    engine: Engine,
}

impl ScriptExecutor {
    /// Create a new executor with all game functions registered
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Safety limits
        engine.set_max_expr_depths(64, 32);
        engine.set_max_operations(10_000);
        engine.set_max_string_size(4096);
        engine.set_max_array_size(1000);
        engine.set_max_map_size(100);
        engine.set_strict_variables(true);

        Self { engine }
    }

    /// Compile a script to an AST for repeated execution
    pub fn compile(&self, source: &str) -> Result<AST, ScriptError> {
        Ok(self.engine.compile(source)?)
    }

    /// Execute a script with the given game state context
    pub fn execute(
        &self,
        ast: &AST,
        state: &GameState,
        tags: &dyn TagProvider,
        rng_seed: u64,
    ) -> Result<ScriptResult, ScriptError> {
        let shared = SharedState::new(rng_seed);
        let mut scope = self.build_scope();

        // Snapshot data for dynamic lookup
        let tag_snapshot = Arc::new(self.snapshot_tags(tags));
        let flag_snapshot = Arc::new(self.snapshot_flags(state));
        let resource_snapshot = Arc::new(self.snapshot_resources(state));
        let reputation_snapshot = Arc::new(self.snapshot_reputations(state));

        // Build engine with all functions registered
        let engine = self.build_engine_with_state(
            shared.clone(),
            tag_snapshot,
            flag_snapshot,
            resource_snapshot,
            reputation_snapshot,
        );

        engine.run_ast_with_scope(&mut scope, ast)?;

        Ok(shared.into_result())
    }

    /// Execute a script from source (compile + run)
    pub fn eval(
        &self,
        source: &str,
        state: &GameState,
        tags: &dyn TagProvider,
        rng_seed: u64,
    ) -> Result<ScriptResult, ScriptError> {
        let ast = self.compile(source)?;
        self.execute(&ast, state, tags, rng_seed)
    }

    /// Evaluate a Rhai expression as a boolean condition.
    ///
    /// This is used for evaluating choice conditions like `resource("credits") >= 100`.
    /// Unlike `eval()`, this returns just the boolean result without collecting effects.
    pub fn eval_condition(
        &self,
        source: &str,
        state: &GameState,
        tags: &dyn TagProvider,
    ) -> Result<bool, ScriptError> {
        // Snapshot data for dynamic lookup
        let tag_snapshot = Arc::new(self.snapshot_tags(tags));
        let flag_snapshot = Arc::new(self.snapshot_flags(state));
        let resource_snapshot = Arc::new(self.snapshot_resources(state));
        let reputation_snapshot = Arc::new(self.snapshot_reputations(state));

        // Build engine with lookup functions only (no effect functions needed for conditions)
        let engine = self.build_condition_engine(
            tag_snapshot,
            flag_snapshot,
            resource_snapshot,
            reputation_snapshot,
        );

        let result: bool = engine.eval(source)?;
        Ok(result)
    }

    /// Build an engine with only lookup functions for condition evaluation.
    fn build_condition_engine(
        &self,
        tag_snapshot: Arc<std::collections::HashMap<String, bool>>,
        flag_snapshot: Arc<std::collections::HashMap<String, Dynamic>>,
        resource_snapshot: Arc<std::collections::HashMap<String, i64>>,
        reputation_snapshot: Arc<std::collections::HashMap<String, i64>>,
    ) -> Engine {
        let mut engine = Engine::new();

        // Copy safety settings
        engine.set_max_expr_depths(64, 32);
        engine.set_max_operations(1_000); // Fewer ops needed for conditions
        engine.set_max_string_size(1024);
        engine.set_strict_variables(false);

        // === Tag Lookup ===
        let tags = tag_snapshot.clone();
        engine.register_fn("has_tag", move |category: ImmutableString, tag: ImmutableString| -> bool {
            let key = format!("{}:{}", category, tag);
            tags.get(&key).copied().unwrap_or(false)
        });

        // === Flag Lookup ===
        let flags = flag_snapshot.clone();
        engine.register_fn("flag", move |name: ImmutableString| -> Dynamic {
            flags.get(name.as_str()).cloned().unwrap_or(Dynamic::UNIT)
        });

        let flags = flag_snapshot.clone();
        engine.register_fn("flag_bool", move |name: ImmutableString| -> bool {
            flags
                .get(name.as_str())
                .and_then(|d| d.as_bool().ok())
                .unwrap_or(false)
        });

        // === Resource Lookup ===
        let resources = resource_snapshot.clone();
        engine.register_fn("resource", move |name: ImmutableString| -> i64 {
            resources.get(name.as_str()).copied().unwrap_or(0)
        });

        // === Reputation Lookup ===
        let reps = reputation_snapshot.clone();
        engine.register_fn("reputation", move |faction: ImmutableString| -> i64 {
            reps.get(faction.as_str()).copied().unwrap_or(0)
        });

        engine
    }

    /// Build an empty scope for script execution.
    /// Note: All game state access is through registered functions (resource(), has_tag(), etc.)
    /// rather than scope variables, because scope variables aren't visible at compile time.
    fn build_scope(&self) -> Scope<'static> {
        Scope::new()
    }

    /// Snapshot tag checks into a lookup map for dynamic has_tag()
    fn snapshot_tags(&self, tags: &dyn TagProvider) -> std::collections::HashMap<String, bool> {
        let mut map = std::collections::HashMap::new();

        // Snapshot all common tag combinations
        let categories = ["ship", "crew", "cargo"];
        let ship_tags = ["sensor", "combat", "stealth", "cargo", "speed", "defense"];
        let crew_tags = ["engineering", "medical", "combat", "social", "navigation"];
        let cargo_tags = ["volatile", "contraband", "organic", "tech", "artifact"];

        for cat in categories {
            let tag_list: &[&str] = match cat {
                "ship" => &ship_tags,
                "crew" => &crew_tags,
                "cargo" => &cargo_tags,
                _ => continue,
            };

            for tag in tag_list {
                let key = format!("{}:{}", cat, tag);
                let has = tags.has_tag(&TagCategoryId::new(cat), &TagId::new(*tag));
                map.insert(key, has);
            }
        }

        map
    }

    /// Snapshot flag values for dynamic flag()
    fn snapshot_flags(&self, state: &GameState) -> std::collections::HashMap<String, Dynamic> {
        let mut map = std::collections::HashMap::new();

        // Copy all flags from state
        for (flag_id, value) in &state.flags {
            let dyn_val = match value {
                Value::Bool(b) => Dynamic::from(*b),
                Value::Int(n) => Dynamic::from(*n),
                Value::Float(f) => Dynamic::from(*f),
                Value::String(s) => Dynamic::from(s.to_string()),
                Value::Null => Dynamic::UNIT,
            };
            map.insert(flag_id.as_str().to_string(), dyn_val);
        }

        map
    }

    /// Snapshot resource values for dynamic resource()
    fn snapshot_resources(&self, state: &GameState) -> std::collections::HashMap<String, i64> {
        let mut map = std::collections::HashMap::new();

        for (resource_id, &value) in &state.resources {
            map.insert(resource_id.as_str().to_string(), value);
        }

        map
    }

    /// Snapshot faction reputation values
    fn snapshot_reputations(&self, state: &GameState) -> std::collections::HashMap<String, i64> {
        let mut map = std::collections::HashMap::new();

        for (faction_id, &value) in &state.factions {
            map.insert(faction_id.as_str().to_string(), value);
        }

        map
    }

    /// Create an engine with state-mutating functions registered
    fn build_engine_with_state(
        &self,
        shared: SharedState,
        tag_snapshot: Arc<std::collections::HashMap<String, bool>>,
        flag_snapshot: Arc<std::collections::HashMap<String, Dynamic>>,
        resource_snapshot: Arc<std::collections::HashMap<String, i64>>,
        reputation_snapshot: Arc<std::collections::HashMap<String, i64>>,
    ) -> Engine {
        let mut engine = Engine::new();

        // Copy safety settings
        engine.set_max_expr_depths(64, 32);
        engine.set_max_operations(10_000);
        engine.set_max_string_size(4096);
        engine.set_max_array_size(1000);
        engine.set_max_map_size(100);
        // Note: strict_variables disabled to allow runtime scope constants
        engine.set_strict_variables(false);

        // === Tag Lookup ===
        let tags = tag_snapshot.clone();
        engine.register_fn("has_tag", move |category: ImmutableString, tag: ImmutableString| -> bool {
            let key = format!("{}:{}", category, tag);
            tags.get(&key).copied().unwrap_or(false)
        });

        // === Flag Lookup ===
        let flags = flag_snapshot.clone();
        engine.register_fn("flag", move |name: ImmutableString| -> Dynamic {
            flags.get(name.as_str()).cloned().unwrap_or(Dynamic::UNIT)
        });

        let flags = flag_snapshot.clone();
        engine.register_fn("flag_bool", move |name: ImmutableString| -> bool {
            flags
                .get(name.as_str())
                .and_then(|d| d.as_bool().ok())
                .unwrap_or(false)
        });

        // === Resource Lookup ===
        let resources = resource_snapshot.clone();
        engine.register_fn("resource", move |name: ImmutableString| -> i64 {
            resources.get(name.as_str()).copied().unwrap_or(0)
        });

        // === Reputation Lookup ===
        let reps = reputation_snapshot.clone();
        engine.register_fn("reputation", move |faction: ImmutableString| -> i64 {
            reps.get(faction.as_str()).copied().unwrap_or(0)
        });

        // === RNG Functions ===
        let rng = shared.rng_state.clone();
        engine.register_fn("rng_float", move || -> f64 {
            let mut state = rng.lock();
            *state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (*state >> 11) as f64 / (1u64 << 53) as f64
        });

        let rng = shared.rng_state.clone();
        engine.register_fn("rng_int", move |min: i64, max: i64| -> i64 {
            if min >= max {
                return min;
            }
            let mut state = rng.lock();
            *state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let f = (*state >> 11) as f64 / (1u64 << 53) as f64;
            min + ((max - min) as f64 * f) as i64
        });

        // === Effect Functions ===
        let effects = shared.effects.clone();
        engine.register_fn("damage", move |resource: ImmutableString, amount: i64| {
            effects
                .lock()
                .push(Effect::damage(resource.as_str(), amount));
        });

        let effects = shared.effects.clone();
        engine.register_fn("modify_resource", move |resource: ImmutableString, delta: i64| {
            effects
                .lock()
                .push(Effect::modify_resource(resource.as_str(), delta));
        });

        let effects = shared.effects.clone();
        engine.register_fn("set_resource", move |resource: ImmutableString, value: i64| {
            effects
                .lock()
                .push(Effect::set_resource(resource.as_str(), value));
        });

        let effects = shared.effects.clone();
        engine.register_fn("add_card", move |card_id: ImmutableString| {
            effects.lock().push(Effect::add_card(card_id.as_str()));
        });

        let effects = shared.effects.clone();
        engine.register_fn("remove_cards", move |pattern: ImmutableString| {
            effects
                .lock()
                .push(Effect::remove_cards(pattern.as_str()));
        });

        let effects = shared.effects.clone();
        engine.register_fn("chronicle", move |title: ImmutableString, text: ImmutableString| {
            effects
                .lock()
                .push(Effect::chronicle(title.as_str(), text.as_str()));
        });

        let effects = shared.effects.clone();
        engine.register_fn("modify_reputation", move |faction: ImmutableString, delta: i64| {
            effects
                .lock()
                .push(Effect::modify_reputation(faction.as_str(), delta));
        });

        // === Flag Functions ===
        let effects = shared.effects.clone();
        engine.register_fn("set_flag", move |name: ImmutableString, value: bool| {
            effects
                .lock()
                .push(Effect::set_flag(name.as_str(), Value::Bool(value)));
        });

        let effects = shared.effects.clone();
        engine.register_fn("set_flag_int", move |name: ImmutableString, value: i64| {
            effects
                .lock()
                .push(Effect::set_flag(name.as_str(), Value::Int(value)));
        });

        let effects = shared.effects.clone();
        engine.register_fn("set_flag_str", move |name: ImmutableString, value: ImmutableString| {
            effects.lock().push(Effect::set_flag(
                name.as_str(),
                Value::String(value.as_str().into()),
            ));
        });

        // === Navigation ===
        // Note: "goto" is a reserved keyword in Rhai, so we use "jump_to"
        let goto = shared.goto.clone();
        engine.register_fn("jump_to", move |passage: ImmutableString| {
            *goto.lock() = Some(passage.to_string());
        });

        engine
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
    use engine_core::{GameState, ResourceId};

    struct NoTags;
    impl TagProvider for NoTags {
        fn has_tag(&self, _: &TagCategoryId, _: &TagId) -> bool {
            false
        }
    }

    #[test]
    fn execute_simple_script() {
        let executor = ScriptExecutor::new();
        let state = GameState::default();
        let tags = NoTags;

        let result = executor
            .eval("let x = 1 + 2;", &state, &tags, 12345)
            .unwrap();

        assert!(result.effects.is_empty());
        assert!(result.goto.is_none());
    }

    #[test]
    fn execute_damage_effect() {
        let executor = ScriptExecutor::new();
        let state = GameState::default();
        let tags = NoTags;

        let result = executor
            .eval(r#"damage("hull", 10);"#, &state, &tags, 12345)
            .unwrap();

        assert_eq!(result.effects.len(), 1);
        match &result.effects[0] {
            Effect::Damage { resource, amount } => {
                assert_eq!(resource.as_str(), "hull");
                assert_eq!(*amount, 10);
            }
            _ => panic!("Expected Damage effect"),
        }
    }

    #[test]
    fn execute_rng_deterministic() {
        let executor = ScriptExecutor::new();
        let state = GameState::default();
        let tags = NoTags;

        // Same seed should produce same results
        let result1 = executor
            .eval("let x = rng_float(); x", &state, &tags, 12345)
            .unwrap();
        let result2 = executor
            .eval("let x = rng_float(); x", &state, &tags, 12345)
            .unwrap();

        assert_eq!(result1.rng_state, result2.rng_state);
    }

    #[test]
    fn execute_jump_to() {
        let executor = ScriptExecutor::new();
        let state = GameState::default();
        let tags = NoTags;

        let result = executor
            .eval(r#"jump_to("next_passage");"#, &state, &tags, 12345)
            .unwrap();

        assert_eq!(result.goto.as_deref(), Some("next_passage"));
    }

    #[test]
    fn execute_conditional_with_resources() {
        let executor = ScriptExecutor::new();
        let mut state = GameState::default();
        state.set_resource(ResourceId::new("hull"), 50);
        let tags = NoTags;

        // Use resource() function for dynamic lookup
        let result = executor
            .eval(
                r#"
                if resource("hull") < 60 {
                    damage("hull", 5);
                }
                "#,
                &state,
                &tags,
                12345,
            )
            .unwrap();

        assert_eq!(result.effects.len(), 1);
    }

    #[test]
    fn execute_has_tag() {
        struct CombatTags;
        impl TagProvider for CombatTags {
            fn has_tag(&self, category: &TagCategoryId, tag: &TagId) -> bool {
                category.as_str() == "ship" && tag.as_str() == "combat"
            }
        }

        let executor = ScriptExecutor::new();
        let state = GameState::default();
        let tags = CombatTags;

        let result = executor
            .eval(
                r#"
                if has_tag("ship", "combat") {
                    damage("hull", 10);
                }
                "#,
                &state,
                &tags,
                12345,
            )
            .unwrap();

        assert_eq!(result.effects.len(), 1);
    }

    #[test]
    fn eval_condition_resource_check() {
        let executor = ScriptExecutor::new();
        let mut state = GameState::default();
        state.set_resource(ResourceId::new("credits"), 150);
        let tags = NoTags;

        // Condition should pass (150 >= 100)
        let result = executor
            .eval_condition(r#"resource("credits") >= 100"#, &state, &tags)
            .unwrap();
        assert!(result);

        // Condition should fail (150 >= 200)
        let result = executor
            .eval_condition(r#"resource("credits") >= 200"#, &state, &tags)
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn eval_condition_tag_check() {
        struct ShipCombatTags;
        impl TagProvider for ShipCombatTags {
            fn has_tag(&self, category: &TagCategoryId, tag: &TagId) -> bool {
                category.as_str() == "ship" && tag.as_str() == "combat"
            }
        }

        let executor = ScriptExecutor::new();
        let state = GameState::default();

        // Should pass - has ship.combat tag
        let result = executor
            .eval_condition(r#"has_tag("ship", "combat")"#, &state, &ShipCombatTags)
            .unwrap();
        assert!(result);

        // Should fail - doesn't have crew.engineering tag
        let result = executor
            .eval_condition(r#"has_tag("crew", "engineering")"#, &state, &ShipCombatTags)
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn eval_condition_combined() {
        struct CombinedTags;
        impl TagProvider for CombinedTags {
            fn has_tag(&self, category: &TagCategoryId, tag: &TagId) -> bool {
                category.as_str() == "ship" && tag.as_str() == "combat"
            }
        }

        let executor = ScriptExecutor::new();
        let mut state = GameState::default();
        state.set_resource(ResourceId::new("credits"), 150);

        // Both conditions true
        let result = executor
            .eval_condition(
                r#"resource("credits") >= 100 && has_tag("ship", "combat")"#,
                &state,
                &CombinedTags,
            )
            .unwrap();
        assert!(result);

        // First true, second false
        let result = executor
            .eval_condition(
                r#"resource("credits") >= 100 && has_tag("ship", "stealth")"#,
                &state,
                &CombinedTags,
            )
            .unwrap();
        assert!(!result);
    }
}
