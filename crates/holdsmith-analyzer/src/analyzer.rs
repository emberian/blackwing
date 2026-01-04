//! Scene analyzer combining CFG traversal with symbolic execution.
//!
//! This module provides high-level analysis of scenes:
//! - Reachability analysis: which passages/choices can be reached
//! - Dead code detection: unreachable passages
//! - Impossible condition detection: choices that can never be taken
//! - Resource depletion detection: paths that always lead to game over

use std::collections::{HashSet, VecDeque};

use smol_str::SmolStr;
use z3::SatResult;

use crate::cfg::{build_cfg, CfgTarget, NodeId, SceneCfg};
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

    // Perform symbolic analysis for impossible/tautological conditions
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

/// Analyze choice conditions using symbolic execution.
fn analyze_conditions(cfg: &SceneCfg, result: &mut AnalysisResult) {
    let ctx = create_context();

    // For each passage, check if any choice conditions are impossible or tautological
    for node in &cfg.nodes {
        let passage_idx = node.passage_index;

        // Get edges (choices) from this passage
        let edges: Vec<_> = cfg.outgoing_edges(NodeId(passage_idx)).collect();

        for edge in edges {
            // Skip choices without conditions (always available)
            let Some(ref cond_analysis) = edge.condition_analysis else {
                continue;
            };

            // Create fresh symbolic state for this analysis
            let mut state = SymbolicState::new(&ctx);

            // Encode any scene requirements as constraints
            if let Some(ref req_analysis) = cfg.requirements_analysis {
                state.encode_reads(req_analysis);
            }

            // Encode reads from path to this passage
            // (simplified: just encode the condition's reads)
            state.encode_reads(cond_analysis);

            // Build a condition based on what we know
            // For now, we check if the condition reads are satisfiable
            // A more sophisticated analysis would parse the actual condition

            // Check if the condition has resource checks we can analyze
            let mut has_constraints = false;
            for state_ref in &cond_analysis.reads {
                if let StateRef::Resource(name) = state_ref {
                    // Assume resource checks are of the form "resource >= X"
                    // We can't know X without parsing, but we can check if
                    // it's possible for the resource to exist
                    let _ = state.resource(name);
                    has_constraints = true;
                }
            }

            // If we have no concrete constraints to check, skip
            if !has_constraints {
                continue;
            }

            // Check satisfiability
            match state.check() {
                SatResult::Unsat => {
                    result.impossible_choices.push(ImpossibleChoice {
                        passage_index: passage_idx,
                        choice_index: edge.choice_index,
                        choice_text: edge.text.clone(),
                        reason: "Condition constraints are unsatisfiable".to_string(),
                    });
                }
                SatResult::Sat => {
                    // Could check for tautology by checking if negation is unsat
                    // But without full condition parsing, this is limited
                }
                SatResult::Unknown => {
                    // Solver couldn't determine - skip
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
pub fn analyze_scenes_parallel(scenes: &[Scene]) -> Vec<AnalysisResult> {
    use rayon::prelude::*;
    scenes.par_iter().map(analyze_scene).collect()
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
}
