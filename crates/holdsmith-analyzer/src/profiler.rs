//! Profile-based scene simulation.
//!
//! This module provides headless scene simulation with different exploration
//! strategies. It enables statistical analysis of scene outcomes without
//! requiring a full game runtime.

use std::collections::HashMap;

use smol_str::SmolStr;

use engine_core::{
    Command, ContentRegistry, GameSchema, GameState, Navigation, Scene, SceneId, TagCategoryId,
    TagId, TagProvider,
};
use engine_runtime::{Rng, Runtime};
use engine_script::ScriptExecutor;

/// Empty tag provider for condition evaluation without tags.
struct NoTags;

impl TagProvider for NoTags {
    fn has_tag(&self, _category: &TagCategoryId, _tag: &TagId) -> bool {
        false
    }
}

/// Information about an available choice.
#[derive(Debug, Clone)]
pub struct ChoiceInfo {
    /// Index of the choice
    pub index: usize,
    /// Choice text
    pub text: SmolStr,
    /// Whether the choice is currently available (condition passed)
    pub available: bool,
    /// Target of the choice
    pub target: ChoiceTarget,
}

/// Target of a choice.
#[derive(Debug, Clone, Copy)]
pub enum ChoiceTarget {
    /// Go to another passage
    Passage(usize),
    /// End the scene
    End,
}

/// Current state during simulation.
#[derive(Debug)]
pub struct SimulationState<'a> {
    /// Current game state
    pub game_state: &'a GameState,
    /// Scene being simulated
    pub scene: &'a Scene,
    /// Current passage index
    pub passage_index: usize,
    /// Number of choices made so far
    pub choices_made: usize,
}

/// Outcome of simulating a scene.
#[derive(Debug, Clone)]
pub struct SceneOutcome {
    /// Scene ID
    pub scene_id: SceneId,
    /// Whether the scene completed normally
    pub completed: bool,
    /// Whether the simulation resulted in game over
    pub game_over: bool,
    /// Reason for game over, if any
    pub game_over_reason: Option<String>,
    /// Number of choices made
    pub choices_made: usize,
    /// Final passage index
    pub final_passage: usize,
    /// Path through the scene (passage, choice) pairs
    pub path: Vec<(usize, usize)>,
    /// Resource deltas during the scene
    pub resource_deltas: HashMap<SmolStr, i64>,
    /// Flags set during the scene
    pub flags_set: Vec<SmolStr>,
}

/// Strategy for selecting choices during simulation.
pub trait ExplorationStrategy: Send + Sync {
    /// Select a choice from available options.
    ///
    /// Returns the index of the selected choice.
    fn select_choice(
        &mut self,
        state: &SimulationState,
        choices: &[ChoiceInfo],
        rng: &mut Rng,
    ) -> usize;

    /// Called when a scene completes.
    fn on_scene_complete(&mut self, _outcome: &SceneOutcome) {}

    /// Clone the strategy (for parallel execution).
    fn clone_strategy(&self) -> Box<dyn ExplorationStrategy>;
}

/// Random strategy: selects uniformly at random from available choices.
#[derive(Debug, Clone, Default)]
pub struct RandomStrategy;

impl ExplorationStrategy for RandomStrategy {
    fn select_choice(
        &mut self,
        _state: &SimulationState,
        choices: &[ChoiceInfo],
        rng: &mut Rng,
    ) -> usize {
        let available: Vec<_> = choices.iter().filter(|c| c.available).collect();
        if available.is_empty() {
            0 // Fallback to first choice
        } else {
            let idx = rng.next_u32_range(available.len() as u32) as usize;
            available[idx].index
        }
    }

    fn clone_strategy(&self) -> Box<dyn ExplorationStrategy> {
        Box::new(self.clone())
    }
}

/// First-available strategy: always takes the first available choice.
#[derive(Debug, Clone, Default)]
pub struct FirstAvailableStrategy;

impl ExplorationStrategy for FirstAvailableStrategy {
    fn select_choice(
        &mut self,
        _state: &SimulationState,
        choices: &[ChoiceInfo],
        _rng: &mut Rng,
    ) -> usize {
        choices
            .iter()
            .find(|c| c.available)
            .map(|c| c.index)
            .unwrap_or(0)
    }

    fn clone_strategy(&self) -> Box<dyn ExplorationStrategy> {
        Box::new(self.clone())
    }
}

/// Specific path strategy: follows a predetermined path.
#[derive(Debug, Clone)]
pub struct PathStrategy {
    /// Predetermined choices to make
    pub path: Vec<usize>,
    /// Current position in path
    pub position: usize,
}

impl PathStrategy {
    pub fn new(path: Vec<usize>) -> Self {
        Self { path, position: 0 }
    }
}

impl ExplorationStrategy for PathStrategy {
    fn select_choice(
        &mut self,
        _state: &SimulationState,
        choices: &[ChoiceInfo],
        _rng: &mut Rng,
    ) -> usize {
        if self.position < self.path.len() {
            let choice = self.path[self.position];
            self.position += 1;
            choice.min(choices.len().saturating_sub(1))
        } else {
            // Path exhausted, take first available
            choices
                .iter()
                .find(|c| c.available)
                .map(|c| c.index)
                .unwrap_or(0)
        }
    }

    fn clone_strategy(&self) -> Box<dyn ExplorationStrategy> {
        Box::new(self.clone())
    }
}

/// Statistics from profiling a scene.
#[derive(Debug, Default)]
pub struct ProfileStatistics {
    /// Total number of simulation runs
    pub total_runs: usize,
    /// Number of runs that resulted in game over
    pub game_overs: usize,
    /// Number of runs that completed normally
    pub completions: usize,
    /// Distribution of resource deltas
    pub resource_distributions: HashMap<SmolStr, Distribution>,
    /// Frequency of each path taken
    pub path_frequencies: HashMap<String, usize>,
    /// Average choices per run
    pub avg_choices: f64,
}

/// Simple distribution statistics.
#[derive(Debug, Clone, Default)]
pub struct Distribution {
    pub min: i64,
    pub max: i64,
    pub sum: i64,
    pub count: usize,
}

impl Distribution {
    pub fn add(&mut self, value: i64) {
        if self.count == 0 {
            self.min = value;
            self.max = value;
        } else {
            self.min = self.min.min(value);
            self.max = self.max.max(value);
        }
        self.sum += value;
        self.count += 1;
    }

    pub fn mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum as f64 / self.count as f64
        }
    }
}

/// Scene profiler for simulation-based analysis.
pub struct Profiler<'a> {
    schema: &'a GameSchema,
    registry: &'a dyn ContentRegistry,
    /// Maximum steps per simulation (prevents infinite loops)
    max_steps: usize,
}

impl<'a> Profiler<'a> {
    /// Create a new profiler.
    pub fn new(schema: &'a GameSchema, registry: &'a dyn ContentRegistry) -> Self {
        Self {
            schema,
            registry,
            max_steps: 100,
        }
    }

    /// Set maximum steps per simulation.
    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// Simulate a scene once with the given strategy.
    pub fn simulate_scene(
        &self,
        scene_id: &SceneId,
        strategy: &mut dyn ExplorationStrategy,
        seed: u64,
    ) -> Option<SceneOutcome> {
        let scene = self.registry.get_scene(scene_id)?;

        let mut runtime = Runtime::new(self.schema, self.registry, seed);
        let mut rng = Rng::new(seed);

        let initial_resources = self.snapshot_resources(runtime.state());

        let mut passage_index = 0;
        let mut path = Vec::new();
        let flags_set = Vec::new();
        let mut game_over = false;
        let mut game_over_reason = None;
        let mut completed = false;

        for _ in 0..self.max_steps {
            let passage = match scene.passages.get(passage_index) {
                Some(p) => p,
                None => break,
            };

            // Evaluate available choices
            let choices = self.evaluate_choices(runtime.state(), scene, passage_index);

            if choices.is_empty() {
                break;
            }

            // Select choice using strategy
            let sim_state = SimulationState {
                game_state: runtime.state(),
                scene,
                passage_index,
                choices_made: path.len(),
            };
            let choice_idx = strategy.select_choice(&sim_state, &choices, &mut rng);
            let choice_idx = choice_idx.min(choices.len() - 1);

            path.push((passage_index, choice_idx));

            // Execute choice
            let command = Command::MakeChoice {
                scene_id: scene_id.clone(),
                passage_index,
                choice_index: choice_idx,
            };

            match runtime.dispatch(command) {
                Ok(_) => {}
                Err(e) => {
                    if let engine_runtime::RuntimeError::GameOver { reason } = e {
                        game_over = true;
                        game_over_reason = Some(reason);
                        break;
                    }
                }
            }

            // Determine next passage
            let choice = &passage.choices[choice_idx];
            match choice.next {
                Navigation::Passage(next) => {
                    passage_index = next;
                }
                Navigation::End => {
                    completed = true;
                    break;
                }
            }
        }

        // Calculate resource deltas
        let final_resources = self.snapshot_resources(runtime.state());
        let resource_deltas = self.calculate_deltas(&initial_resources, &final_resources);

        let outcome = SceneOutcome {
            scene_id: scene_id.clone(),
            completed,
            game_over,
            game_over_reason,
            choices_made: path.len(),
            final_passage: passage_index,
            path,
            resource_deltas,
            flags_set,
        };

        strategy.on_scene_complete(&outcome);

        Some(outcome)
    }

    /// Profile a scene with multiple simulation runs.
    pub fn profile_scene(
        &self,
        scene_id: &SceneId,
        strategy: &mut dyn ExplorationStrategy,
        runs: usize,
        base_seed: u64,
    ) -> ProfileStatistics {
        let mut stats = ProfileStatistics::default();
        let mut total_choices = 0usize;

        for i in 0..runs {
            let seed = base_seed.wrapping_add(i as u64);
            let mut strat = strategy.clone_strategy();

            if let Some(outcome) = self.simulate_scene(scene_id, strat.as_mut(), seed) {
                stats.total_runs += 1;

                if outcome.game_over {
                    stats.game_overs += 1;
                } else if outcome.completed {
                    stats.completions += 1;
                }

                total_choices += outcome.choices_made;

                // Track resource distributions
                for (resource, delta) in &outcome.resource_deltas {
                    stats
                        .resource_distributions
                        .entry(resource.clone())
                        .or_default()
                        .add(*delta);
                }

                // Track path frequencies
                let path_sig = outcome
                    .path
                    .iter()
                    .map(|(p, c)| format!("{}:{}", p, c))
                    .collect::<Vec<_>>()
                    .join("->");
                *stats.path_frequencies.entry(path_sig).or_default() += 1;
            }
        }

        if stats.total_runs > 0 {
            stats.avg_choices = total_choices as f64 / stats.total_runs as f64;
        }

        stats
    }

    /// Evaluate which choices are available at a passage.
    fn evaluate_choices(
        &self,
        state: &GameState,
        scene: &Scene,
        passage_index: usize,
    ) -> Vec<ChoiceInfo> {
        let passage = match scene.passages.get(passage_index) {
            Some(p) => p,
            None => return Vec::new(),
        };

        let executor = ScriptExecutor::new();
        let no_tags = NoTags;

        passage
            .choices
            .iter()
            .enumerate()
            .map(|(idx, choice)| {
                let available = if choice.rhai_condition.is_empty() {
                    true
                } else {
                    executor
                        .eval_condition(&choice.rhai_condition, state, &no_tags)
                        .unwrap_or(false)
                };

                ChoiceInfo {
                    index: idx,
                    text: choice.text.clone(),
                    available,
                    target: match choice.next {
                        Navigation::Passage(p) => ChoiceTarget::Passage(p),
                        Navigation::End => ChoiceTarget::End,
                    },
                }
            })
            .collect()
    }

    /// Snapshot current resource values.
    fn snapshot_resources(&self, state: &GameState) -> HashMap<SmolStr, i64> {
        let mut snapshot = HashMap::new();
        for resource in &self.schema.resources {
            snapshot.insert(SmolStr::new(resource.id.as_str()), state.resource(&resource.id));
        }
        snapshot
    }

    /// Calculate deltas between snapshots.
    fn calculate_deltas(
        &self,
        initial: &HashMap<SmolStr, i64>,
        final_: &HashMap<SmolStr, i64>,
    ) -> HashMap<SmolStr, i64> {
        let mut deltas = HashMap::new();
        for (name, initial_val) in initial {
            if let Some(&final_val) = final_.get(name) {
                let delta = final_val - initial_val;
                if delta != 0 {
                    deltas.insert(name.clone(), delta);
                }
            }
        }
        deltas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::*;

    struct TestRegistry {
        scenes: HashMap<SceneId, Scene>,
    }

    impl TestRegistry {
        fn new(scenes: Vec<Scene>) -> Self {
            let mut map = HashMap::new();
            for scene in scenes {
                map.insert(scene.id.clone(), scene);
            }
            Self { scenes: map }
        }
    }

    impl ContentRegistry for TestRegistry {
        fn get_scene(&self, id: &SceneId) -> Option<&Scene> {
            self.scenes.get(id)
        }

        fn get_card(&self, _id: &CardId) -> Option<&CardDef> {
            None
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

    fn test_schema() -> GameSchema {
        GameSchema {
            name: "Test".to_string(),
            version: "1.0".to_string(),
            resources: vec![
                ResourceDef {
                    id: ResourceId::new("credits"),
                    name: "Credits".into(),
                    min: Some(0),
                    max: None,
                    default: 100,
                    on_zero: None,
                },
                ResourceDef {
                    id: ResourceId::new("hull"),
                    name: "Hull".into(),
                    min: Some(0),
                    max: Some(100),
                    default: 100,
                    on_zero: Some(OnZeroBehavior::GameOver {
                        reason: "destroyed".to_string(),
                    }),
                },
            ],
            tag_categories: vec![],
            card_types: vec![],
            contexts: vec![],
            slot_types: vec![],
            location_statuses: vec![],
            initial_state: InitialStateDef::default(),
        }
    }

    fn test_scene() -> Scene {
        Scene {
            id: SceneId::new("test_scene"),
            title: "Test".into(),
            tags: Tags::default(),
            context: None,
            weight: 10,
            cooldown: 0,
            rhai_requirements: Default::default(),
            passages: vec![
                Passage {
                    text: "Start".into(),
                    rhai_on_enter: None,
                    choices: vec![
                        Choice {
                            text: "Safe".into(),
                            next: Navigation::End,
                            rhai_condition: Default::default(),
                            rhai_effects: Default::default(),
                        },
                        Choice {
                            text: "Risky".into(),
                            next: Navigation::End,
                            rhai_condition: Default::default(),
                            rhai_effects: r#"damage("hull", 50)"#.into(),
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn test_random_strategy() {
        let schema = test_schema();
        let registry = TestRegistry::new(vec![test_scene()]);
        let profiler = Profiler::new(&schema, &registry);

        let mut strategy = RandomStrategy;
        let scene_id = SceneId::new("test_scene");

        let outcome = profiler.simulate_scene(&scene_id, &mut strategy, 42);
        assert!(outcome.is_some());

        let outcome = outcome.unwrap();
        assert!(outcome.completed);
        assert!(!outcome.game_over);
    }

    #[test]
    fn test_path_strategy() {
        let schema = test_schema();
        let registry = TestRegistry::new(vec![test_scene()]);
        let profiler = Profiler::new(&schema, &registry);

        // Always take the safe path (choice 0)
        let mut strategy = PathStrategy::new(vec![0]);
        let scene_id = SceneId::new("test_scene");

        let outcome = profiler.simulate_scene(&scene_id, &mut strategy, 42).unwrap();
        assert!(outcome.completed);
        assert!(outcome.resource_deltas.get("hull").is_none()); // No hull damage

        // Always take the risky path (choice 1)
        let mut strategy = PathStrategy::new(vec![1]);
        let outcome = profiler.simulate_scene(&scene_id, &mut strategy, 42).unwrap();
        assert!(outcome.completed);
        assert_eq!(outcome.resource_deltas.get("hull"), Some(&-50));
    }

    #[test]
    fn test_profile_statistics() {
        let schema = test_schema();
        let registry = TestRegistry::new(vec![test_scene()]);
        let profiler = Profiler::new(&schema, &registry);

        let mut strategy = RandomStrategy;
        let scene_id = SceneId::new("test_scene");

        let stats = profiler.profile_scene(&scene_id, &mut strategy, 100, 0);

        assert_eq!(stats.total_runs, 100);
        assert!(stats.completions > 0);
        // Some runs should take safe path, some risky
        assert!(stats.path_frequencies.len() <= 2);
    }

    #[test]
    fn test_distribution() {
        let mut dist = Distribution::default();
        dist.add(10);
        dist.add(20);
        dist.add(30);

        assert_eq!(dist.min, 10);
        assert_eq!(dist.max, 30);
        assert_eq!(dist.count, 3);
        assert!((dist.mean() - 20.0).abs() < 0.001);
    }
}
