//! Path-sensitive analysis for scene execution.
//!
//! This module performs symbolic execution along CFG paths:
//! - Enumerate all paths through the scene (with depth limit)
//! - Accumulate constraints (conditions) and effects along paths
//! - Detect death paths (paths where critical resources can reach zero)
//! - Calculate resource bounds at scene exit

use std::collections::{HashMap, HashSet};

use smol_str::SmolStr;
use z3::ast::Bool;

use crate::cfg::{CfgTarget, SceneCfg};
use crate::condition_encoder::{ConditionEncoder, EncodedCondition};
use crate::symbolic::{create_context, SymbolicState};
use engine_script::AnalyzedEffect;

/// A step in an execution path.
#[derive(Debug, Clone)]
pub struct PathStep {
    /// Passage index
    pub passage: usize,
    /// Choice index taken from this passage (None for final passage)
    pub choice: Option<usize>,
}

/// A complete execution path through the scene.
#[derive(Debug, Clone)]
pub struct ExecutionPath {
    /// Sequence of steps in the path
    pub steps: Vec<PathStep>,
    /// Whether the path is fully deterministic (no RNG)
    pub is_deterministic: bool,
    /// Whether this path ends the scene (vs looping back)
    pub reaches_end: bool,
}

impl ExecutionPath {
    /// Get a signature string for this path.
    pub fn signature(&self) -> String {
        self.steps
            .iter()
            .map(|s| {
                match s.choice {
                    Some(c) => format!("{}:{}", s.passage, c),
                    None => format!("{}", s.passage),
                }
            })
            .collect::<Vec<_>>()
            .join("->")
    }
}

/// Information about a death path.
#[derive(Debug, Clone)]
pub struct DeathPath {
    /// The path that leads to death
    pub path: ExecutionPath,
    /// The resource that reaches zero
    pub resource: SmolStr,
    /// Minimum possible value at the end of path (if deterministic)
    pub min_value: Option<i64>,
    /// Whether the death is certain or just possible
    pub is_certain: bool,
}

/// Resource bounds at scene exit.
#[derive(Debug, Clone, Default)]
pub struct ResourceBounds {
    /// Minimum possible deltas for each resource
    pub min_deltas: HashMap<SmolStr, i64>,
    /// Maximum possible deltas for each resource
    pub max_deltas: HashMap<SmolStr, i64>,
}

/// Warning about RNG affecting analysis.
#[derive(Debug, Clone)]
pub struct RngWarning {
    /// Path where RNG is used
    pub path: ExecutionPath,
    /// Description of the RNG usage
    pub description: String,
}

/// Result of path analysis.
#[derive(Debug, Default)]
pub struct PathAnalysisResult {
    /// All enumerated paths
    pub paths: Vec<ExecutionPath>,
    /// Paths that can lead to death
    pub death_paths: Vec<DeathPath>,
    /// Resource bounds at scene exits
    pub exit_bounds: ResourceBounds,
    /// RNG warnings
    pub rng_warnings: Vec<RngWarning>,
    /// Whether analysis was bounded (hit depth limit)
    pub was_bounded: bool,
}

/// Path analyzer for a scene CFG.
pub struct PathAnalyzer<'a> {
    cfg: &'a SceneCfg,
    /// Resources that cause game over when reaching zero
    death_resources: HashSet<SmolStr>,
    /// Maximum depth for path enumeration
    max_depth: usize,
}

impl<'a> PathAnalyzer<'a> {
    /// Create a new path analyzer.
    ///
    /// # Arguments
    /// - `cfg`: The scene CFG to analyze
    /// - `death_resources`: Set of resource names that cause game over when zero
    /// - `max_depth`: Maximum path length to enumerate
    pub fn new(
        cfg: &'a SceneCfg,
        death_resources: HashSet<SmolStr>,
        max_depth: usize,
    ) -> Self {
        Self {
            cfg,
            death_resources,
            max_depth,
        }
    }

    /// Perform path analysis.
    pub fn analyze(&self) -> PathAnalysisResult {
        let mut result = PathAnalysisResult::default();

        // Enumerate all paths
        let (paths, was_bounded) = self.enumerate_paths();
        result.paths = paths;
        result.was_bounded = was_bounded;

        // Analyze each path for death conditions and bounds
        for path in &result.paths {
            // Check for death paths
            if let Some(death) = self.check_death_path(path) {
                result.death_paths.push(death);
            }

            // Track RNG warnings
            if !path.is_deterministic {
                result.rng_warnings.push(RngWarning {
                    path: path.clone(),
                    description: "Path contains RNG, bounds are conservative".to_string(),
                });
            }
        }

        // Calculate exit bounds across all paths
        result.exit_bounds = self.calculate_bounds(&result.paths);

        result
    }

    /// Enumerate all paths through the CFG up to max_depth.
    fn enumerate_paths(&self) -> (Vec<ExecutionPath>, bool) {
        let mut paths = Vec::new();
        let mut was_bounded = false;

        // DFS stack: (current_node, current_path, visited_set)
        let mut stack = vec![(
            self.cfg.entry,
            Vec::new(),
            HashSet::new(),
            true, // is_deterministic
        )];

        while let Some((node, mut path, mut visited, is_deterministic)) = stack.pop() {
            // Check depth limit
            if path.len() >= self.max_depth {
                was_bounded = true;
                // Still record the partial path
                paths.push(ExecutionPath {
                    steps: path,
                    is_deterministic,
                    reaches_end: false,
                });
                continue;
            }

            // Check for cycles
            if visited.contains(&node) {
                // Cyclic path - record and stop
                paths.push(ExecutionPath {
                    steps: path,
                    is_deterministic,
                    reaches_end: false,
                });
                continue;
            }

            visited.insert(node);

            // Get outgoing edges
            let edges: Vec<_> = self.cfg.outgoing_edges(node).collect();

            if edges.is_empty() {
                // Dead end (shouldn't happen in well-formed CFG)
                path.push(PathStep {
                    passage: node.0,
                    choice: None,
                });
                paths.push(ExecutionPath {
                    steps: path,
                    is_deterministic,
                    reaches_end: false,
                });
                continue;
            }

            // Explore each choice
            for edge in edges {
                let mut new_path = path.clone();
                new_path.push(PathStep {
                    passage: node.0,
                    choice: Some(edge.choice_index),
                });

                // Check if this edge uses RNG
                let edge_deterministic = edge
                    .condition_analysis
                    .as_ref()
                    .map(|a| !a.uses_rng)
                    .unwrap_or(true)
                    && edge
                        .effects_analysis
                        .as_ref()
                        .map(|a| !a.uses_rng)
                        .unwrap_or(true);

                let new_deterministic = is_deterministic && edge_deterministic;

                match edge.to {
                    CfgTarget::End => {
                        // Path ends here
                        paths.push(ExecutionPath {
                            steps: new_path,
                            is_deterministic: new_deterministic,
                            reaches_end: true,
                        });
                    }
                    CfgTarget::Node(next) => {
                        // Continue DFS
                        stack.push((next, new_path, visited.clone(), new_deterministic));
                    }
                }
            }
        }

        (paths, was_bounded)
    }

    /// Check if a path can lead to death (resource reaching zero).
    fn check_death_path(&self, path: &ExecutionPath) -> Option<DeathPath> {
        if self.death_resources.is_empty() {
            return None;
        }

        // Track deltas concretely (could use Z3 for symbolic analysis in future)
        let mut resource_deltas: HashMap<SmolStr, i64> = HashMap::new();

        // Walk the path and accumulate effects
        for step in &path.steps {
            let node = &self.cfg.nodes[step.passage];

            // Apply on-enter effects
            if let Some(ref analysis) = node.on_enter_analysis {
                self.accumulate_effects(&analysis.writes, &mut resource_deltas);
            }

            // Apply choice effects
            if let Some(choice_idx) = step.choice {
                if let Some(edge) = self.cfg.edges.iter().find(|e| {
                    e.from.0 == step.passage && e.choice_index == choice_idx
                }) {
                    if let Some(ref analysis) = edge.effects_analysis {
                        self.accumulate_effects(&analysis.writes, &mut resource_deltas);
                    }
                }
            }
        }

        // Check each death resource
        for resource in &self.death_resources {
            if let Some(&delta) = resource_deltas.get(resource) {
                // If delta is significantly negative, this could be a death path
                // For precise analysis we'd need to know the starting value,
                // but we can report if there's any negative delta
                if delta < 0 {
                    return Some(DeathPath {
                        path: path.clone(),
                        resource: resource.clone(),
                        min_value: Some(delta),
                        is_certain: path.is_deterministic,
                    });
                }
            }
        }

        None
    }

    /// Accumulate effects into resource deltas.
    fn accumulate_effects(
        &self,
        effects: &[AnalyzedEffect],
        deltas: &mut HashMap<SmolStr, i64>,
    ) {
        for effect in effects {
            match effect {
                AnalyzedEffect::Damage { resource, amount } => {
                    if let Some(amt) = amount {
                        *deltas.entry(resource.clone()).or_default() -= amt;
                    }
                }
                AnalyzedEffect::ModifyResource { resource, delta } => {
                    if let Some(d) = delta {
                        *deltas.entry(resource.clone()).or_default() += d;
                    }
                }
                AnalyzedEffect::SetResource { resource, value } => {
                    // Set is tricky - we can't easily combine with deltas
                    // For now, treat as replacing previous delta
                    if let Some(v) = value {
                        deltas.insert(resource.clone(), *v);
                    }
                }
                _ => {}
            }
        }
    }

    /// Calculate resource bounds across all paths.
    fn calculate_bounds(&self, paths: &[ExecutionPath]) -> ResourceBounds {
        let mut bounds = ResourceBounds::default();

        // First pass: collect all deltas per path
        let mut path_deltas: Vec<HashMap<SmolStr, i64>> = Vec::new();
        let mut all_resources: HashSet<SmolStr> = HashSet::new();

        for path in paths {
            if !path.reaches_end {
                continue; // Only consider paths that complete the scene
            }

            let mut deltas: HashMap<SmolStr, i64> = HashMap::new();

            // Walk path and accumulate
            for step in &path.steps {
                let node = &self.cfg.nodes[step.passage];

                if let Some(ref analysis) = node.on_enter_analysis {
                    self.accumulate_effects(&analysis.writes, &mut deltas);
                }

                if let Some(choice_idx) = step.choice {
                    if let Some(edge) = self.cfg.edges.iter().find(|e| {
                        e.from.0 == step.passage && e.choice_index == choice_idx
                    }) {
                        if let Some(ref analysis) = edge.effects_analysis {
                            self.accumulate_effects(&analysis.writes, &mut deltas);
                        }
                    }
                }
            }

            // Track all resources seen
            all_resources.extend(deltas.keys().cloned());
            path_deltas.push(deltas);
        }

        // Second pass: update bounds, treating missing resources as delta 0
        for deltas in &path_deltas {
            for resource in &all_resources {
                let delta = *deltas.get(resource).unwrap_or(&0);

                let min = bounds.min_deltas.entry(resource.clone()).or_insert(delta);
                if delta < *min {
                    *min = delta;
                }

                let max = bounds.max_deltas.entry(resource.clone()).or_insert(delta);
                if delta > *max {
                    *max = delta;
                }
            }
        }

        bounds
    }
}

/// Check if a specific path is feasible given scene requirements.
pub fn check_path_feasibility(cfg: &SceneCfg, path: &ExecutionPath) -> PathFeasibility {
    let ctx = create_context();
    let mut state = SymbolicState::new(&ctx);

    // Encode scene requirements
    if !cfg.requirements_source.is_empty() {
        let mut encoder = ConditionEncoder::new(&ctx, &mut state);
        if let EncodedCondition::Bool(constraint) = encoder.encode_script(&cfg.requirements_source)
        {
            state.assert_constraint(&constraint);
        }
    }

    // Encode each choice condition along the path
    let mut conditions = Vec::new();
    for step in &path.steps {
        if let Some(choice_idx) = step.choice {
            if let Some(edge) = cfg.edges.iter().find(|e| {
                e.from.0 == step.passage && e.choice_index == choice_idx
            }) {
                if !edge.condition_source.is_empty() {
                    let mut encoder = ConditionEncoder::new(&ctx, &mut state);
                    match encoder.encode_script(&edge.condition_source) {
                        EncodedCondition::Bool(cond) => conditions.push(cond),
                        EncodedCondition::NonDeterministic => {
                            return PathFeasibility::NonDeterministic;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Check if all conditions can be satisfied together
    if conditions.is_empty() {
        return PathFeasibility::Feasible;
    }

    let refs: Vec<&Bool> = conditions.iter().collect();
    let combined = Bool::and(&ctx, &refs);

    if state.is_unsat(&combined) {
        PathFeasibility::Infeasible
    } else if state.is_valid(&combined) {
        PathFeasibility::AlwaysTaken
    } else {
        PathFeasibility::Feasible
    }
}

/// Result of path feasibility check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathFeasibility {
    /// Path can be taken under some conditions
    Feasible,
    /// Path cannot be taken (conditions are contradictory)
    Infeasible,
    /// Path is always taken (conditions are tautological)
    AlwaysTaken,
    /// Path depends on RNG
    NonDeterministic,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::build_cfg;
    use engine_core::*;

    fn test_scene() -> Scene {
        Scene {
            id: SceneId::new("test"),
            title: "Test Scene".into(),
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
                            text: "Safe path".into(),
                            next: Navigation::Passage(1),
                            rhai_condition: Default::default(),
                            rhai_effects: Default::default(),
                        },
                        Choice {
                            text: "Risky path".into(),
                            next: Navigation::Passage(2),
                            rhai_condition: Default::default(),
                            rhai_effects: r#"damage("hull", 50)"#.into(),
                        },
                    ],
                },
                Passage {
                    text: "Safe end".into(),
                    rhai_on_enter: None,
                    choices: vec![Choice {
                        text: "Done".into(),
                        next: Navigation::End,
                        rhai_condition: Default::default(),
                        rhai_effects: Default::default(),
                    }],
                },
                Passage {
                    text: "Risky end".into(),
                    rhai_on_enter: None,
                    choices: vec![Choice {
                        text: "Survive".into(),
                        next: Navigation::End,
                        rhai_condition: Default::default(),
                        rhai_effects: Default::default(),
                    }],
                },
            ],
        }
    }

    #[test]
    fn test_enumerate_paths() {
        let scene = test_scene();
        let cfg = build_cfg(&scene);

        let analyzer = PathAnalyzer::new(&cfg, HashSet::new(), 10);
        let result = analyzer.analyze();

        // Should have 2 paths: Start->Safe->End and Start->Risky->End
        assert_eq!(result.paths.len(), 2);
        assert!(result.paths.iter().all(|p| p.reaches_end));
        assert!(!result.was_bounded);
    }

    #[test]
    fn test_death_path_detection() {
        let scene = test_scene();
        let cfg = build_cfg(&scene);

        let mut death_resources = HashSet::new();
        death_resources.insert(SmolStr::new("hull"));

        let analyzer = PathAnalyzer::new(&cfg, death_resources, 10);
        let result = analyzer.analyze();

        // Should detect the risky path as potentially deadly
        assert_eq!(result.death_paths.len(), 1);
        assert_eq!(result.death_paths[0].resource.as_str(), "hull");
    }

    #[test]
    fn test_resource_bounds() {
        let scene = test_scene();
        let cfg = build_cfg(&scene);

        let analyzer = PathAnalyzer::new(&cfg, HashSet::new(), 10);
        let result = analyzer.analyze();

        // Hull delta should range from -50 (risky) to 0 (safe)
        assert_eq!(result.exit_bounds.min_deltas.get("hull"), Some(&-50));
        assert_eq!(result.exit_bounds.max_deltas.get("hull"), Some(&0));
    }

    #[test]
    fn test_depth_limit() {
        // Create a scene with a loop
        let scene = Scene {
            id: SceneId::new("loop"),
            title: "Loop Scene".into(),
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
                            text: "Loop".into(),
                            next: Navigation::Passage(1),
                            rhai_condition: Default::default(),
                            rhai_effects: Default::default(),
                        },
                        Choice {
                            text: "Exit".into(),
                            next: Navigation::End,
                            rhai_condition: Default::default(),
                            rhai_effects: Default::default(),
                        },
                    ],
                },
                Passage {
                    text: "Middle".into(),
                    rhai_on_enter: None,
                    choices: vec![Choice {
                        text: "Back".into(),
                        next: Navigation::Passage(0),
                        rhai_condition: Default::default(),
                        rhai_effects: Default::default(),
                    }],
                },
            ],
        };

        let cfg = build_cfg(&scene);
        let analyzer = PathAnalyzer::new(&cfg, HashSet::new(), 5);
        let result = analyzer.analyze();

        // Should have multiple paths due to loop exploration before hitting cycle detection
        assert!(!result.paths.is_empty());
    }

    #[test]
    fn test_path_feasibility() {
        let scene = Scene {
            id: SceneId::new("conditional"),
            title: "Conditional".into(),
            tags: Tags::default(),
            context: None,
            weight: 10,
            cooldown: 0,
            rhai_requirements: r#"resource("credits") >= 100"#.into(),
            passages: vec![Passage {
                text: "Start".into(),
                rhai_on_enter: None,
                choices: vec![
                    Choice {
                        text: "Rich path".into(),
                        next: Navigation::End,
                        rhai_condition: r#"resource("credits") >= 200"#.into(),
                        rhai_effects: Default::default(),
                    },
                    Choice {
                        text: "Poor path".into(),
                        next: Navigation::End,
                        rhai_condition: r#"resource("credits") < 50"#.into(),
                        rhai_effects: Default::default(),
                    },
                ],
            }],
        };

        let cfg = build_cfg(&scene);

        // Poor path should be infeasible (requires credits < 50 but scene requires >= 100)
        let poor_path = ExecutionPath {
            steps: vec![PathStep {
                passage: 0,
                choice: Some(1),
            }],
            is_deterministic: true,
            reaches_end: true,
        };

        let feasibility = check_path_feasibility(&cfg, &poor_path);
        assert_eq!(feasibility, PathFeasibility::Infeasible);
    }
}
