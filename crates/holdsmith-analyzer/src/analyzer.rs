//! Scene analyzer combining CFG traversal with symbolic execution.
//!
//! This module provides high-level analysis of scenes:
//! - Reachability analysis: which passages/choices can be reached
//! - Dead code detection: unreachable passages
//! - Impossible condition detection: choices that can never be taken (requires `z3` feature)
//! - Resource depletion detection: paths that always lead to game over

use std::collections::{HashSet, VecDeque};

use smol_str::SmolStr;


use crate::cfg::{build_cfg, CfgTarget, SceneCfg};
#[cfg(feature = "z3")]
use crate::cfg::NodeId;
#[cfg(feature = "z3")]
use crate::symbolic::{create_context, SymbolicState};
use engine_core::Scene;
use engine_script::StateRef;

/// Result of analyzing a scene.
#[derive(Debug, Default)]
pub struct AnalysisResult {
    /// Scene ID
    pub scene_id: SmolStr,
    /// Passages that are reachable from entry
    pub reachable_passages: HashSet<usize>,
    /// Passages that are unreachable (dead code)
    pub unreachable_passages: Vec<usize>,
    /// Choices that can never be taken (impossible conditions)
    pub impossible_choices: Vec<ImpossibleChoice>,
    /// Choices that are always available (tautological conditions)
    pub tautological_choices: Vec<TautologicalChoice>,
    /// Resources that are read but might not be initialized
    pub uninitialized_reads: Vec<UninitializedRead>,
    /// Whether the scene uses RNG (non-deterministic)
    pub uses_rng: bool,
    /// All resources read by this scene
    pub resources_read: HashSet<SmolStr>,
    /// All resources written by this scene
    pub resources_written: HashSet<SmolStr>,
    /// All flags read by this scene
    pub flags_read: HashSet<SmolStr>,
    /// All flags written by this scene
    pub flags_written: HashSet<SmolStr>,
    /// Errors encountered during analysis
    pub errors: Vec<String>,
}

/// A choice that can never be taken.
#[derive(Debug, Clone)]
pub struct ImpossibleChoice {
    pub passage_index: usize,
    pub choice_index: usize,
    pub choice_text: SmolStr,
    pub reason: String,
}

/// A choice whose condition is always true.
#[derive(Debug, Clone)]
pub struct TautologicalChoice {
    pub passage_index: usize,
    pub choice_index: usize,
    pub choice_text: SmolStr,
}

/// A resource/flag read that might not be initialized.
#[derive(Debug, Clone)]
pub struct UninitializedRead {
    pub location: ReadLocation,
    pub state_ref: SmolStr,
    pub kind: StateRefKind,
}

#[derive(Debug, Clone)]
pub enum ReadLocation {
    SceneRequirements,
    PassageOnEnter { passage_index: usize },
    ChoiceCondition { passage_index: usize, choice_index: usize },
    ChoiceEffects { passage_index: usize, choice_index: usize },
}

#[derive(Debug, Clone, Copy)]
pub enum StateRefKind {
    Resource,
    Flag,
    Reputation,
}

/// Analyze a scene for potential issues.
pub fn analyze_scene(scene: &Scene) -> AnalysisResult {
    let cfg = build_cfg(scene);
    analyze_cfg(&cfg)
}

/// Analyze a pre-built CFG.
pub fn analyze_cfg(cfg: &SceneCfg) -> AnalysisResult {
    let mut result = AnalysisResult {
        scene_id: cfg.scene_id.clone(),
        uses_rng: cfg.uses_rng,
        ..Default::default()
    };

    // Copy any CFG errors
    for err in &cfg.errors {
        result.errors.push(format!("{:?}: {}", err.location, err.message));
    }

    // Collect all state references
    collect_state_refs(cfg, &mut result);

    // Perform reachability analysis
    analyze_reachability(cfg, &mut result);

    // Perform symbolic analysis for impossible/tautological conditions (requires Z3)
    #[cfg(feature = "z3")]
    analyze_conditions(cfg, &mut result);

    result
}

/// Collect all state references (resources, flags) from the CFG.
fn collect_state_refs(cfg: &SceneCfg, result: &mut AnalysisResult) {
    for state_ref in cfg.all_reads() {
        match state_ref {
            StateRef::Resource(name) => {
                result.resources_read.insert(name.clone());
            }
            StateRef::Flag(name) => {
                result.flags_read.insert(name.clone());
            }
            StateRef::Reputation(faction) => {
                result.resources_read.insert(format!("rep:{}", faction).into());
            }
            _ => {}
        }
    }

    for effect in cfg.all_writes() {
        match effect {
            engine_script::AnalyzedEffect::Damage { resource, .. }
            | engine_script::AnalyzedEffect::ModifyResource { resource, .. }
            | engine_script::AnalyzedEffect::SetResource { resource, .. } => {
                result.resources_written.insert(resource.clone());
            }
            engine_script::AnalyzedEffect::SetFlag { flag } => {
                result.flags_written.insert(flag.clone());
            }
            engine_script::AnalyzedEffect::ModifyReputation { faction, .. } => {
                result.resources_written.insert(format!("rep:{}", faction).into());
            }
            _ => {}
        }
    }
}

/// Perform BFS reachability analysis from entry node.
fn analyze_reachability(cfg: &SceneCfg, result: &mut AnalysisResult) {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(cfg.entry);

    while let Some(node_id) = queue.pop_front() {
        if visited.contains(&node_id) {
            continue;
        }
        visited.insert(node_id);
        result.reachable_passages.insert(node_id.0);

        // Add all reachable neighbors
        for edge in cfg.outgoing_edges(node_id) {
            if let CfgTarget::Node(target) = edge.to {
                if !visited.contains(&target) {
                    queue.push_back(target);
                }
            }
        }
    }

    // Find unreachable passages
    for (idx, _) in cfg.nodes.iter().enumerate() {
        if !result.reachable_passages.contains(&idx) {
            result.unreachable_passages.push(idx);
        }
    }
}

/// Analyze choice conditions using symbolic execution and Z3.
///
/// This function:
/// 1. Encodes scene requirements as preconditions
/// 2. Applies effects from incoming edges (path-sensitive analysis)
/// 3. Applies on_enter effects for the current passage
/// 4. Encodes each choice condition as a Z3 formula
/// 5. Checks if conditions are impossible (unsatisfiable) or tautological (always true)
#[cfg(feature = "z3")]
fn analyze_conditions(cfg: &SceneCfg, result: &mut AnalysisResult) {
    use crate::cfg::CfgEdge;
    use crate::condition_encoder::{ConditionEncoder, EncodedCondition};
    use std::collections::HashMap;

    let ctx = create_context();

    // Build map of incoming edges for each passage (for path-sensitive analysis)
    let mut incoming_edges: HashMap<usize, Vec<&CfgEdge>> = HashMap::new();
    for edge in &cfg.edges {
        if let CfgTarget::Node(target) = edge.to {
            incoming_edges.entry(target.0).or_default().push(edge);
        }
    }

    // For each passage, check if any choice conditions are impossible or tautological
    for node in &cfg.nodes {
        let passage_idx = node.passage_index;

        // Get edges (choices) from this passage
        let edges: Vec<_> = cfg.outgoing_edges(NodeId(passage_idx)).collect();

        for edge in edges {
            // Skip choices without conditions (empty conditions are always available)
            if edge.condition_source.is_empty() {
                continue;
            }

            // Skip RNG-dependent conditions (can't analyze statically)
            if let Some(ref cond_analysis) = edge.condition_analysis {
                if cond_analysis.uses_rng {
                    continue;
                }
            }

            // Create fresh symbolic state for this analysis
            let mut state = SymbolicState::new(&ctx);

            // Encode scene requirements as preconditions
            if !cfg.requirements_source.is_empty() {
                let mut encoder = ConditionEncoder::new(&ctx, &mut state);
                if let EncodedCondition::Bool(req_constraint) = encoder.encode_script(&cfg.requirements_source) {
                    state.assert_constraint(&req_constraint);
                }
            }

            // Path-sensitive analysis: apply effects from incoming edges
            // For passages with a unique incoming edge, we can precisely track the effects.
            // For passages with multiple incoming edges, we take a conservative approach.
            if passage_idx > 0 {
                if let Some(incoming) = incoming_edges.get(&passage_idx) {
                    if incoming.len() == 1 {
                        // Single incoming edge - apply its effects precisely
                        let incoming_edge = incoming[0];
                        if let Some(ref effects_analysis) = incoming_edge.effects_analysis {
                            state.apply_writes(effects_analysis);
                        }
                        // Also apply on_enter effects from the source passage
                        if let Some(source_node) = cfg.node(incoming_edge.from) {
                            if let Some(ref source_on_enter) = source_node.on_enter_analysis {
                                state.apply_writes(source_on_enter);
                            }
                        }
                    }
                    // For multiple incoming edges, we'd need to:
                    // - Find common effects (intersection)
                    // - Or analyze each path separately
                    // For now, we don't apply any accumulated effects (conservative)
                }
            }

            // Apply on_enter effects for the current passage
            if let Some(ref on_enter) = node.on_enter_analysis {
                state.apply_writes(on_enter);
            }

            // Encode the choice condition
            let mut encoder = ConditionEncoder::new(&ctx, &mut state);
            let condition = encoder.encode_script(&edge.condition_source);

            match condition {
                EncodedCondition::Bool(cond_constraint) => {
                    // Check if condition is impossible (unsat given preconditions)
                    if state.is_unsat(&cond_constraint) {
                        result.impossible_choices.push(ImpossibleChoice {
                            passage_index: passage_idx,
                            choice_index: edge.choice_index,
                            choice_text: edge.text.clone(),
                            reason: "Condition is unsatisfiable given scene requirements".to_string(),
                        });
                    }
                    // Check if condition is a tautology (always true given preconditions)
                    else if state.is_valid(&cond_constraint) {
                        result.tautological_choices.push(TautologicalChoice {
                            passage_index: passage_idx,
                            choice_index: edge.choice_index,
                            choice_text: edge.text.clone(),
                        });
                    }
                }
                EncodedCondition::NonDeterministic => {
                    // RNG-dependent condition - already skipped above, but handle gracefully
                }
                EncodedCondition::Int(_) => {
                    // Condition evaluated to an integer, not a boolean - likely a bug in the scene
                    result.errors.push(format!(
                        "Choice condition in passage {} choice {} evaluates to integer, not boolean",
                        passage_idx, edge.choice_index
                    ));
                }
                EncodedCondition::Unknown(_reason) => {
                    // Could not encode - log but don't treat as error (might be complex pattern)
                    // This is fine for now; we can extend the encoder to handle more patterns
                }
            }
        }
    }
}

/// Statistics about a scene.
#[derive(Debug, Default)]
pub struct SceneStats {
    pub total_passages: usize,
    pub total_choices: usize,
    pub reachable_passages: usize,
    pub uses_rng: bool,
    pub unique_resources_read: usize,
    pub unique_resources_written: usize,
    pub unique_flags_read: usize,
    pub unique_flags_written: usize,
}

impl From<&AnalysisResult> for SceneStats {
    fn from(result: &AnalysisResult) -> Self {
        Self {
            reachable_passages: result.reachable_passages.len(),
            uses_rng: result.uses_rng,
            unique_resources_read: result.resources_read.len(),
            unique_resources_written: result.resources_written.len(),
            unique_flags_read: result.flags_read.len(),
            unique_flags_written: result.flags_written.len(),
            ..Default::default()
        }
    }
}

/// Analyze multiple scenes and aggregate results.
pub fn analyze_scenes(scenes: &[Scene]) -> Vec<AnalysisResult> {
    scenes.iter().map(analyze_scene).collect()
}

/// Analyze multiple scenes in parallel using rayon.
#[cfg(feature = "parallel")]
pub fn analyze_scenes_parallel(scenes: &[Scene]) -> Vec<AnalysisResult> {
    use rayon::prelude::*;
    scenes.par_iter().map(analyze_scene).collect()
}

/// Fallback for when parallel feature is disabled (e.g., WASM).
#[cfg(not(feature = "parallel"))]
pub fn analyze_scenes_parallel(scenes: &[Scene]) -> Vec<AnalysisResult> {
    analyze_scenes(scenes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::*;

    fn simple_scene() -> Scene {
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
                            text: "Go to middle".into(),
                            next: Navigation::Passage(1),
                            rhai_condition: Default::default(),
                            rhai_effects: Default::default(),
                        },
                        Choice {
                            text: "Skip to end".into(),
                            next: Navigation::Passage(2),
                            rhai_condition: Default::default(),
                            rhai_effects: Default::default(),
                        },
                    ],
                },
                Passage {
                    text: "Middle".into(),
                    rhai_on_enter: None,
                    choices: vec![
                        Choice {
                            text: "Continue".into(),
                            next: Navigation::Passage(2),
                            rhai_condition: Default::default(),
                            rhai_effects: Default::default(),
                        },
                    ],
                },
                Passage {
                    text: "End".into(),
                    rhai_on_enter: None,
                    choices: vec![
                        Choice {
                            text: "Done".into(),
                            next: Navigation::End,
                            rhai_condition: Default::default(),
                            rhai_effects: Default::default(),
                        },
                    ],
                },
                // Unreachable passage
                Passage {
                    text: "Unreachable".into(),
                    rhai_on_enter: None,
                    choices: vec![
                        Choice {
                            text: "Can't get here".into(),
                            next: Navigation::End,
                            rhai_condition: Default::default(),
                            rhai_effects: Default::default(),
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn test_reachability() {
        let scene = simple_scene();
        let result = analyze_scene(&scene);

        // Passages 0, 1, 2 are reachable
        assert!(result.reachable_passages.contains(&0));
        assert!(result.reachable_passages.contains(&1));
        assert!(result.reachable_passages.contains(&2));

        // Passage 3 is unreachable
        assert!(!result.reachable_passages.contains(&3));
        assert!(result.unreachable_passages.contains(&3));
    }

    #[test]
    fn test_state_ref_collection() {
        let mut scene = simple_scene();
        scene.passages[0].rhai_on_enter = Some(r#"
            if resource("credits") >= 100 {
                damage("hull", 10);
            }
        "#.into());
        scene.passages[0].choices[0].rhai_condition = r#"flag("visited")"#.into();
        scene.passages[0].choices[0].rhai_effects = r#"set_flag("completed", true)"#.into();

        let result = analyze_scene(&scene);

        assert!(result.resources_read.contains("credits"));
        assert!(result.resources_written.contains("hull"));
        assert!(result.flags_read.contains("visited"));
        assert!(result.flags_written.contains("completed"));
    }

    #[test]
    fn test_scene_stats() {
        let scene = simple_scene();
        let result = analyze_scene(&scene);
        let stats = SceneStats::from(&result);

        assert_eq!(stats.reachable_passages, 3); // 0, 1, 2 are reachable
        assert!(!stats.uses_rng);
    }

    #[cfg(feature = "z3")]
    #[test]
    fn test_impossible_choice_detection() {
        // Scene requires credits >= 100, but choice requires credits < 50
        // This should be detected as impossible
        let scene = Scene {
            id: SceneId::new("test_impossible"),
            title: "Impossible Test".into(),
            tags: Tags::default(),
            context: None,
            weight: 10,
            cooldown: 0,
            rhai_requirements: r#"resource("credits") >= 100"#.into(),
            passages: vec![
                Passage {
                    text: "Start".into(),
                    rhai_on_enter: None,
                    choices: vec![
                        Choice {
                            text: "Impossible choice".into(),
                            next: Navigation::End,
                            rhai_condition: r#"resource("credits") < 50"#.into(),
                            rhai_effects: Default::default(),
                        },
                        Choice {
                            text: "Normal choice".into(),
                            next: Navigation::End,
                            rhai_condition: Default::default(),
                            rhai_effects: Default::default(),
                        },
                    ],
                },
            ],
        };

        let result = analyze_scene(&scene);

        // Should detect the impossible choice
        assert_eq!(result.impossible_choices.len(), 1);
        assert_eq!(result.impossible_choices[0].choice_text.as_str(), "Impossible choice");
    }

    #[cfg(feature = "z3")]
    #[test]
    fn test_tautological_choice_detection() {
        // Scene requires credits >= 100, choice requires credits >= 50
        // The choice condition is always true given the scene requirements
        let scene = Scene {
            id: SceneId::new("test_tautology"),
            title: "Tautology Test".into(),
            tags: Tags::default(),
            context: None,
            weight: 10,
            cooldown: 0,
            rhai_requirements: r#"resource("credits") >= 100"#.into(),
            passages: vec![
                Passage {
                    text: "Start".into(),
                    rhai_on_enter: None,
                    choices: vec![
                        Choice {
                            text: "Always available".into(),
                            next: Navigation::End,
                            rhai_condition: r#"resource("credits") >= 50"#.into(),
                            rhai_effects: Default::default(),
                        },
                    ],
                },
            ],
        };

        let result = analyze_scene(&scene);

        // Should detect the tautological choice
        assert_eq!(result.tautological_choices.len(), 1);
        assert_eq!(result.tautological_choices[0].choice_text.as_str(), "Always available");
    }

    // ==================== BUG DEMONSTRATION TESTS ====================
    // These tests demonstrate bugs in the current analysis that need fixing.

    #[cfg(feature = "z3")]
    #[test]
    fn test_bug_on_enter_effects_not_tracked() {
        // BUG: on_enter effects are not tracked when analyzing choice conditions.
        //
        // Scenario:
        // - Scene requires credits >= 100
        // - Passage on_enter sets credits to 0
        // - Choice requires credits >= 50
        //
        // Current (buggy) behavior: Reports choice as tautological (100 >= 50)
        // Correct behavior: Should report choice as IMPOSSIBLE (0 >= 50 is false)
        let scene = Scene {
            id: SceneId::new("test_on_enter_bug"),
            title: "On-Enter Bug".into(),
            tags: Tags::default(),
            context: None,
            weight: 10,
            cooldown: 0,
            rhai_requirements: r#"resource("credits") >= 100"#.into(),
            passages: vec![
                Passage {
                    text: "Start".into(),
                    // This effect sets credits to 0, but analyzer ignores it!
                    rhai_on_enter: Some(r#"set_resource("credits", 0)"#.into()),
                    choices: vec![
                        Choice {
                            text: "Should be impossible".into(),
                            next: Navigation::End,
                            // After on_enter, credits = 0, so this should be impossible
                            rhai_condition: r#"resource("credits") >= 50"#.into(),
                            rhai_effects: Default::default(),
                        },
                    ],
                },
            ],
        };

        let result = analyze_scene(&scene);

        // After applying on_enter effects, the analyzer should see that:
        // - Scene requires credits >= 100, so we enter with credits >= 100
        // - on_enter sets credits = 0
        // - Choice requires credits >= 50, but credits = 0 after on_enter
        // Therefore the choice is IMPOSSIBLE
        assert_eq!(result.tautological_choices.len(), 0,
            "Choice should not be tautological - on_enter sets credits to 0");
        assert_eq!(result.impossible_choices.len(), 1,
            "Choice should be impossible - credits = 0 after on_enter, but condition requires >= 50");
        assert_eq!(result.impossible_choices[0].choice_text.as_str(), "Should be impossible");
    }

    #[cfg(feature = "z3")]
    #[test]
    fn test_path_sensitive_effect_tracking() {
        // Test: Effects from a prior choice should be tracked when analyzing subsequent passages.
        //
        // Scenario:
        // - Scene requires credits >= 0 (establishes baseline)
        // - Passage 0, Choice 0: gives +200 credits, goes to passage 1
        // - Passage 1, Choice 0: requires credits >= 100
        //
        // With path-sensitive analysis:
        // - We know credits >= 0 from requirements
        // - Choice effects add +200, so credits >= 200 when entering passage 1
        // - Choice condition credits >= 100 is always true (200 >= 100)
        // - Therefore the choice should be detected as tautological
        let scene = Scene {
            id: SceneId::new("test_path_sensitive"),
            title: "Path Sensitive Test".into(),
            tags: Tags::default(),
            context: None,
            weight: 10,
            cooldown: 0,
            rhai_requirements: r#"resource("credits") >= 0"#.into(), // Establish baseline
            passages: vec![
                Passage {
                    text: "Passage 0".into(),
                    rhai_on_enter: None,
                    choices: vec![
                        Choice {
                            text: "Get rich".into(),
                            next: Navigation::Passage(1),
                            rhai_condition: Default::default(),
                            // This gives +200 credits
                            rhai_effects: r#"modify_resource("credits", 200)"#.into(),
                        },
                    ],
                },
                Passage {
                    text: "Passage 1".into(),
                    rhai_on_enter: None,
                    choices: vec![
                        Choice {
                            text: "Spend some".into(),
                            next: Navigation::End,
                            // After getting +200 with initial >= 0, we have >= 200
                            // So >= 100 is always true
                            rhai_condition: r#"resource("credits") >= 100"#.into(),
                            rhai_effects: Default::default(),
                        },
                    ],
                },
            ],
        };

        let result = analyze_scene(&scene);

        // With path-sensitive analysis, the choice should be tautological:
        // - Initial credits >= 0 (from requirements)
        // - After +200, credits >= 200
        // - Condition credits >= 100 is always true when credits >= 200
        assert_eq!(result.impossible_choices.len(), 0,
            "Choice should not be impossible");
        assert_eq!(result.tautological_choices.len(), 1,
            "Choice should be tautological - credits >= 200 after effect, so >= 100 is always true");
        assert_eq!(result.tautological_choices[0].choice_text.as_str(), "Spend some");
    }
}
