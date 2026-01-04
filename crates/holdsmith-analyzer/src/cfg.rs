//! Scene-level control flow graph builder.
//!
//! This module builds a CFG from a compiled Scene, where:
//! - Nodes represent passages
//! - Edges represent choice transitions with optional conditions and effects
//! - Each node/edge carries analyzed script information (reads, writes, RNG usage)

use engine_core::{Navigation, Scene};
use engine_script::{analyze_script, ScriptAnalysis};
use smol_str::SmolStr;

/// A unique identifier for a CFG node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// A unique identifier for a CFG edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeId(pub usize);

/// A node in the scene control flow graph (represents a passage).
#[derive(Debug, Clone)]
pub struct CfgNode {
    /// Index into the scene's passages array
    pub passage_index: usize,
    /// Raw Rhai on-enter source (for counterexamples)
    pub on_enter_source: SmolStr,
    /// Analyzed on-enter effects (from passage's rhai_on_enter)
    pub on_enter_analysis: Option<ScriptAnalysis>,
    /// Whether this is an exit node (no outgoing edges to non-END)
    pub is_exit: bool,
}

/// An edge in the scene control flow graph (represents a choice).
#[derive(Debug, Clone)]
pub struct CfgEdge {
    /// Source passage index
    pub from: NodeId,
    /// Target (either a passage index or END)
    pub to: CfgTarget,
    /// The choice index in the source passage
    pub choice_index: usize,
    /// Choice text
    pub text: SmolStr,
    /// Raw Rhai condition source (for Z3 encoding)
    pub condition_source: SmolStr,
    /// Analyzed condition (from choice's rhai_condition)
    pub condition_analysis: Option<ScriptAnalysis>,
    /// Raw Rhai effects source (for Z3 encoding)
    pub effects_source: SmolStr,
    /// Analyzed effects (from choice's rhai_effects)
    pub effects_analysis: Option<ScriptAnalysis>,
}

/// The target of a CFG edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfgTarget {
    /// Transitions to another passage
    Node(NodeId),
    /// Ends the scene
    End,
}

/// Scene-level control flow graph.
#[derive(Debug)]
pub struct SceneCfg {
    /// Scene ID for reference
    pub scene_id: SmolStr,
    /// All nodes (passages) in the CFG
    pub nodes: Vec<CfgNode>,
    /// All edges (choices) in the CFG
    pub edges: Vec<CfgEdge>,
    /// Entry node (always passage 0)
    pub entry: NodeId,
    /// Exit nodes (passages where all choices lead to END)
    pub exits: Vec<NodeId>,
    /// Raw Rhai requirements source (for Z3 encoding)
    pub requirements_source: SmolStr,
    /// Scene requirement analysis (from scene's rhai_requirements)
    pub requirements_analysis: Option<ScriptAnalysis>,
    /// Whether any script in this scene uses RNG
    pub uses_rng: bool,
    /// Errors encountered during analysis
    pub errors: Vec<CfgError>,
}

/// Error during CFG construction.
#[derive(Debug, Clone)]
pub struct CfgError {
    pub location: CfgLocation,
    pub message: String,
}

/// Location in the scene where an error occurred.
#[derive(Debug, Clone)]
pub enum CfgLocation {
    SceneRequirements,
    Passage { index: usize },
    Choice { passage_index: usize, choice_index: usize },
}

impl SceneCfg {
    /// Get all outgoing edges from a node.
    pub fn outgoing_edges(&self, node: NodeId) -> impl Iterator<Item = &CfgEdge> {
        self.edges.iter().filter(move |e| e.from == node)
    }

    /// Get the node for a given passage index.
    pub fn node(&self, id: NodeId) -> Option<&CfgNode> {
        self.nodes.get(id.0)
    }

    /// Check if the scene is deterministic (no RNG usage anywhere).
    pub fn is_deterministic(&self) -> bool {
        !self.uses_rng
    }

    /// Get all state reads across the entire scene.
    pub fn all_reads(&self) -> impl Iterator<Item = &engine_script::StateRef> {
        let req_reads = self.requirements_analysis.iter().flat_map(|a| a.reads.iter());
        let node_reads = self.nodes.iter().flat_map(|n| {
            n.on_enter_analysis.iter().flat_map(|a| a.reads.iter())
        });
        let edge_reads = self.edges.iter().flat_map(|e| {
            let cond_reads = e.condition_analysis.iter().flat_map(|a| a.reads.iter());
            let eff_reads = e.effects_analysis.iter().flat_map(|a| a.reads.iter());
            cond_reads.chain(eff_reads)
        });
        req_reads.chain(node_reads).chain(edge_reads)
    }

    /// Get all state writes across the entire scene.
    pub fn all_writes(&self) -> impl Iterator<Item = &engine_script::AnalyzedEffect> {
        let node_writes = self.nodes.iter().flat_map(|n| {
            n.on_enter_analysis.iter().flat_map(|a| a.writes.iter())
        });
        let edge_writes = self.edges.iter().flat_map(|e| {
            e.effects_analysis.iter().flat_map(|a| a.writes.iter())
        });
        node_writes.chain(edge_writes)
    }
}

/// Build a control flow graph from a compiled scene.
pub fn build_cfg(scene: &Scene) -> SceneCfg {
    let mut nodes = Vec::with_capacity(scene.passages.len());
    let mut edges = Vec::new();
    let mut errors = Vec::new();
    let mut uses_rng = false;

    // Analyze scene requirements (empty string = no requirements)
    let requirements_analysis = if scene.rhai_requirements.is_empty() {
        None
    } else {
        match analyze_script(&scene.rhai_requirements) {
            Ok(analysis) => {
                if analysis.uses_rng {
                    uses_rng = true;
                }
                Some(analysis)
            }
            Err(e) => {
                errors.push(CfgError {
                    location: CfgLocation::SceneRequirements,
                    message: e.to_string(),
                });
                None
            }
        }
    };

    // Build nodes from passages
    for (passage_index, passage) in scene.passages.iter().enumerate() {
        let on_enter_analysis = passage.rhai_on_enter.as_ref().and_then(|script| {
            match analyze_script(script.as_str()) {
                Ok(analysis) => {
                    if analysis.uses_rng {
                        uses_rng = true;
                    }
                    Some(analysis)
                }
                Err(e) => {
                    errors.push(CfgError {
                        location: CfgLocation::Passage { index: passage_index },
                        message: e.to_string(),
                    });
                    None
                }
            }
        });

        // Check if all choices lead to END (making this an exit)
        let is_exit = passage.choices.iter().all(|c| matches!(c.next, Navigation::End));

        // Get the raw on_enter source for counterexamples
        let on_enter_source = passage
            .rhai_on_enter
            .as_ref()
            .map(|s| s.clone())
            .unwrap_or_default();

        nodes.push(CfgNode {
            passage_index,
            on_enter_source,
            on_enter_analysis,
            is_exit,
        });
    }

    // Build edges from choices
    for (passage_index, passage) in scene.passages.iter().enumerate() {
        let from = NodeId(passage_index);

        for (choice_index, choice) in passage.choices.iter().enumerate() {
            // Analyze condition (empty string = always true, no analysis)
            let condition_analysis = if choice.rhai_condition.is_empty() {
                None
            } else {
                match analyze_script(&choice.rhai_condition) {
                    Ok(analysis) => {
                        if analysis.uses_rng {
                            uses_rng = true;
                        }
                        Some(analysis)
                    }
                    Err(e) => {
                        errors.push(CfgError {
                            location: CfgLocation::Choice { passage_index, choice_index },
                            message: format!("condition: {}", e),
                        });
                        None
                    }
                }
            };

            // Analyze effects (empty string = no effects)
            let effects_analysis = if choice.rhai_effects.is_empty() {
                None
            } else {
                match analyze_script(&choice.rhai_effects) {
                    Ok(analysis) => {
                        if analysis.uses_rng {
                            uses_rng = true;
                        }
                        Some(analysis)
                    }
                    Err(e) => {
                        errors.push(CfgError {
                            location: CfgLocation::Choice { passage_index, choice_index },
                            message: format!("effects: {}", e),
                        });
                        None
                    }
                }
            };

            let to = match choice.next {
                Navigation::Passage(idx) => CfgTarget::Node(NodeId(idx)),
                Navigation::End => CfgTarget::End,
            };

            edges.push(CfgEdge {
                from,
                to,
                choice_index,
                text: choice.text.clone(),
                condition_source: choice.rhai_condition.clone(),
                condition_analysis,
                effects_source: choice.rhai_effects.clone(),
                effects_analysis,
            });
        }
    }

    // Collect exit nodes
    let exits: Vec<NodeId> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.is_exit)
        .map(|(i, _)| NodeId(i))
        .collect();

    SceneCfg {
        scene_id: scene.id.as_str().into(),
        nodes,
        edges,
        entry: NodeId(0),
        exits,
        requirements_source: scene.rhai_requirements.clone(),
        requirements_analysis,
        uses_rng,
        errors,
    }
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
            rhai_requirements: r#"resource("credits") >= 100"#.into(),
            passages: vec![
                Passage {
                    text: "Introduction".into(),
                    rhai_on_enter: Some(r#"set_flag("started", true)"#.into()),
                    choices: vec![
                        Choice {
                            text: "Pay credits".into(),
                            next: Navigation::Passage(1),
                            rhai_condition: r#"resource("credits") >= 50"#.into(),
                            rhai_effects: r#"modify_resource("credits", -50)"#.into(),
                        },
                        Choice {
                            text: "Leave".into(),
                            next: Navigation::End,
                            rhai_condition: Default::default(),
                            rhai_effects: Default::default(),
                        },
                    ],
                },
                Passage {
                    text: "Success!".into(),
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
            ],
        }
    }

    #[test]
    fn test_build_cfg() {
        let scene = simple_scene();
        let cfg = build_cfg(&scene);

        assert_eq!(cfg.scene_id.as_str(), "test");
        assert_eq!(cfg.nodes.len(), 2);
        assert_eq!(cfg.edges.len(), 3);
        assert_eq!(cfg.entry.0, 0);
        assert!(cfg.errors.is_empty());
    }

    #[test]
    fn test_cfg_exits() {
        let scene = simple_scene();
        let cfg = build_cfg(&scene);

        // Passage 1 is an exit (only choice leads to END)
        assert!(cfg.exits.contains(&NodeId(1)));
        // Passage 0 has choices leading to both another passage and END
        assert!(!cfg.exits.contains(&NodeId(0)));
    }

    #[test]
    fn test_requirements_analysis() {
        let scene = simple_scene();
        let cfg = build_cfg(&scene);

        let req = cfg.requirements_analysis.as_ref().unwrap();
        assert!(req.reads_resource("credits"));
    }

    #[test]
    fn test_passage_analysis() {
        let scene = simple_scene();
        let cfg = build_cfg(&scene);

        let node0 = &cfg.nodes[0];
        let analysis = node0.on_enter_analysis.as_ref().unwrap();
        assert!(analysis.writes.iter().any(|e| {
            matches!(e, engine_script::AnalyzedEffect::SetFlag { flag } if flag == "started")
        }));
    }

    #[test]
    fn test_choice_analysis() {
        let scene = simple_scene();
        let cfg = build_cfg(&scene);

        // Find the "Pay credits" edge
        let pay_edge = cfg.edges.iter().find(|e| e.text == "Pay credits").unwrap();

        // Check condition analysis
        let cond = pay_edge.condition_analysis.as_ref().unwrap();
        assert!(cond.reads_resource("credits"));

        // Check effects analysis
        let eff = pay_edge.effects_analysis.as_ref().unwrap();
        assert!(eff.writes_resource("credits"));
    }

    #[test]
    fn test_deterministic() {
        let scene = simple_scene();
        let cfg = build_cfg(&scene);

        assert!(cfg.is_deterministic());
    }

    #[test]
    fn test_non_deterministic() {
        let mut scene = simple_scene();
        scene.passages[0].rhai_on_enter = Some(r#"let x = rng_float(); damage("hull", 10)"#.into());

        let cfg = build_cfg(&scene);
        assert!(!cfg.is_deterministic());
    }
}
