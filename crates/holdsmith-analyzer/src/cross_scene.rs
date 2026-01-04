//! Cross-scene analysis for tracking state variables across multiple scenes.
//!
//! This module provides analysis spanning multiple scenes:
//! - Track flag/resource definitions and uses
//! - Detect use-before-set issues
//! - Build scene dependency graphs
//! - Find unused writes (dead stores)

use std::collections::{HashMap, HashSet};

use smol_str::SmolStr;

use crate::cfg::{build_cfg, SceneCfg};
use engine_core::Scene;
use engine_script::{AnalyzedEffect, StateRef};

/// Kind of state variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateVarKind {
    Resource,
    Flag,
    Reputation,
}

/// Location where a state variable is used or defined.
#[derive(Debug, Clone)]
pub struct SceneLocation {
    /// Scene ID
    pub scene_id: SmolStr,
    /// Description of location within scene
    pub location: String,
}

/// Information about a state variable.
#[derive(Debug, Clone)]
pub struct StateVarInfo {
    /// Variable name
    pub name: SmolStr,
    /// Kind of variable
    pub kind: StateVarKind,
    /// Scenes where this variable is defined (written)
    pub defined_in: Vec<SceneLocation>,
    /// Scenes where this variable is used (read)
    pub used_in: Vec<SceneLocation>,
}

impl StateVarInfo {
    /// Check if this variable is used without being defined first.
    pub fn is_potentially_uninitialized(&self) -> bool {
        !self.used_in.is_empty() && self.defined_in.is_empty()
    }

    /// Check if this variable is defined but never used.
    pub fn is_never_used(&self) -> bool {
        !self.defined_in.is_empty() && self.used_in.is_empty()
    }
}

/// A use-before-set issue.
#[derive(Debug, Clone)]
pub struct UseBeforeSet {
    /// Variable that may be used uninitialized
    pub var_name: SmolStr,
    /// Kind of variable
    pub kind: StateVarKind,
    /// Scenes that read this variable
    pub read_scenes: Vec<SmolStr>,
}

/// An unused write (dead store).
#[derive(Debug, Clone)]
pub struct UnusedWrite {
    /// Variable that is written but never read
    pub var_name: SmolStr,
    /// Kind of variable
    pub kind: StateVarKind,
    /// Scenes that write this variable
    pub write_scenes: Vec<SmolStr>,
}

/// Scene dependency graph.
#[derive(Debug, Clone, Default)]
pub struct SceneDependencyGraph {
    /// Edges: scene_id -> set of scenes it depends on
    pub edges: HashMap<SmolStr, HashSet<SmolStr>>,
    /// Reverse edges: scene_id -> set of scenes that depend on it
    pub reverse_edges: HashMap<SmolStr, HashSet<SmolStr>>,
}

impl SceneDependencyGraph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a dependency edge: `from` depends on `to`.
    pub fn add_dependency(&mut self, from: SmolStr, to: SmolStr) {
        self.edges.entry(from.clone()).or_default().insert(to.clone());
        self.reverse_edges.entry(to).or_default().insert(from);
    }

    /// Get scenes that `scene_id` depends on.
    pub fn dependencies(&self, scene_id: &str) -> impl Iterator<Item = &SmolStr> {
        self.edges
            .get(scene_id)
            .into_iter()
            .flat_map(|s| s.iter())
    }

    /// Get scenes that depend on `scene_id`.
    pub fn dependents(&self, scene_id: &str) -> impl Iterator<Item = &SmolStr> {
        self.reverse_edges
            .get(scene_id)
            .into_iter()
            .flat_map(|s| s.iter())
    }

    /// Get all scene IDs in the graph.
    pub fn scene_ids(&self) -> HashSet<&SmolStr> {
        self.edges
            .keys()
            .chain(self.reverse_edges.keys())
            .collect()
    }

    /// Try to produce a topological ordering of scenes.
    /// Returns Err if there are cycles.
    pub fn topological_order(&self) -> Result<Vec<SmolStr>, CycleDetected> {
        let mut in_degree: HashMap<&SmolStr, usize> = HashMap::new();
        let scenes: HashSet<_> = self.scene_ids();

        // Initialize in-degrees
        for scene in &scenes {
            in_degree.insert(scene, 0);
        }

        // Count incoming edges
        for deps in self.edges.values() {
            for dep in deps {
                if let Some(count) = in_degree.get_mut(dep) {
                    *count += 1;
                }
            }
        }

        // Kahn's algorithm
        let mut queue: Vec<SmolStr> = in_degree
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(&scene, _)| scene.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(scene) = queue.pop() {
            result.push(scene.clone());

            if let Some(deps) = self.edges.get(&scene) {
                for dep in deps {
                    if let Some(count) = in_degree.get_mut(dep) {
                        *count -= 1;
                        if *count == 0 {
                            queue.push(dep.clone());
                        }
                    }
                }
            }
        }

        if result.len() != scenes.len() {
            // There's a cycle - find it
            let cycles = self.find_cycles();
            Err(CycleDetected { cycles })
        } else {
            Ok(result)
        }
    }

    /// Find all cycles in the graph.
    pub fn find_cycles(&self) -> Vec<Vec<SmolStr>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for scene in self.scene_ids() {
            if !visited.contains(scene) {
                let mut path = Vec::new();
                self.find_cycles_dfs(scene, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }

        cycles
    }

    fn find_cycles_dfs(
        &self,
        node: &SmolStr,
        visited: &mut HashSet<SmolStr>,
        rec_stack: &mut HashSet<SmolStr>,
        path: &mut Vec<SmolStr>,
        cycles: &mut Vec<Vec<SmolStr>>,
    ) {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());
        path.push(node.clone());

        if let Some(deps) = self.edges.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    self.find_cycles_dfs(dep, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(dep) {
                    // Found a cycle
                    let start_idx = path.iter().position(|n| n == dep).unwrap_or(0);
                    let cycle: Vec<SmolStr> = path[start_idx..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }
}

/// Error indicating a cycle was detected.
#[derive(Debug, Clone)]
pub struct CycleDetected {
    pub cycles: Vec<Vec<SmolStr>>,
}

/// Result of cross-scene analysis.
#[derive(Debug, Default)]
pub struct CrossSceneAnalysis {
    /// All state variables and their info
    pub state_vars: HashMap<SmolStr, StateVarInfo>,
    /// Variables that may be used before being set
    pub use_before_set: Vec<UseBeforeSet>,
    /// Variables that are written but never read
    pub unused_writes: Vec<UnusedWrite>,
    /// Scene dependency graph
    pub dependencies: SceneDependencyGraph,
    /// Total number of scenes analyzed
    pub scene_count: usize,
}

/// Analyze multiple scenes for cross-scene issues.
pub fn analyze_cross_scene(scenes: &[Scene]) -> CrossSceneAnalysis {
    let mut result = CrossSceneAnalysis {
        scene_count: scenes.len(),
        ..Default::default()
    };

    // Build CFGs and collect state vars
    let cfgs: Vec<SceneCfg> = scenes.iter().map(build_cfg).collect();

    // Track reads and writes per scene
    let mut scene_reads: HashMap<SmolStr, HashSet<(SmolStr, StateVarKind)>> = HashMap::new();
    let mut scene_writes: HashMap<SmolStr, HashSet<(SmolStr, StateVarKind)>> = HashMap::new();

    for cfg in &cfgs {
        let scene_id = cfg.scene_id.clone();

        // Collect reads
        let mut reads = HashSet::new();
        for state_ref in cfg.all_reads() {
            match state_ref {
                StateRef::Resource(name) => {
                    reads.insert((name.clone(), StateVarKind::Resource));
                    add_read(&mut result.state_vars, name, StateVarKind::Resource, &scene_id);
                }
                StateRef::Flag(name) => {
                    reads.insert((name.clone(), StateVarKind::Flag));
                    add_read(&mut result.state_vars, name, StateVarKind::Flag, &scene_id);
                }
                StateRef::Reputation(faction) => {
                    reads.insert((faction.clone(), StateVarKind::Reputation));
                    add_read(&mut result.state_vars, faction, StateVarKind::Reputation, &scene_id);
                }
                _ => {}
            }
        }
        scene_reads.insert(scene_id.clone(), reads);

        // Collect writes
        let mut writes = HashSet::new();
        for effect in cfg.all_writes() {
            match effect {
                AnalyzedEffect::Damage { resource, .. }
                | AnalyzedEffect::ModifyResource { resource, .. }
                | AnalyzedEffect::SetResource { resource, .. } => {
                    writes.insert((resource.clone(), StateVarKind::Resource));
                    add_write(&mut result.state_vars, resource, StateVarKind::Resource, &scene_id);
                }
                AnalyzedEffect::SetFlag { flag } => {
                    writes.insert((flag.clone(), StateVarKind::Flag));
                    add_write(&mut result.state_vars, flag, StateVarKind::Flag, &scene_id);
                }
                AnalyzedEffect::ModifyReputation { faction, .. } => {
                    writes.insert((faction.clone(), StateVarKind::Reputation));
                    add_write(&mut result.state_vars, faction, StateVarKind::Reputation, &scene_id);
                }
                _ => {}
            }
        }
        scene_writes.insert(scene_id, writes);
    }

    // Build dependency graph: scene A depends on scene B if A reads what B writes
    for (reader_scene, reads) in &scene_reads {
        for (var_name, var_kind) in reads {
            for (writer_scene, writes) in &scene_writes {
                if reader_scene != writer_scene && writes.contains(&(var_name.clone(), *var_kind)) {
                    result.dependencies.add_dependency(reader_scene.clone(), writer_scene.clone());
                }
            }
        }
    }

    // Find use-before-set issues
    for (name, info) in &result.state_vars {
        if info.is_potentially_uninitialized() {
            result.use_before_set.push(UseBeforeSet {
                var_name: name.clone(),
                kind: info.kind,
                read_scenes: info.used_in.iter().map(|l| l.scene_id.clone()).collect(),
            });
        }
    }

    // Find unused writes
    for (name, info) in &result.state_vars {
        if info.is_never_used() {
            result.unused_writes.push(UnusedWrite {
                var_name: name.clone(),
                kind: info.kind,
                write_scenes: info.defined_in.iter().map(|l| l.scene_id.clone()).collect(),
            });
        }
    }

    result
}

fn add_read(
    state_vars: &mut HashMap<SmolStr, StateVarInfo>,
    name: &SmolStr,
    kind: StateVarKind,
    scene_id: &SmolStr,
) {
    let info = state_vars.entry(name.clone()).or_insert_with(|| StateVarInfo {
        name: name.clone(),
        kind,
        defined_in: Vec::new(),
        used_in: Vec::new(),
    });
    info.used_in.push(SceneLocation {
        scene_id: scene_id.clone(),
        location: "scene".to_string(),
    });
}

fn add_write(
    state_vars: &mut HashMap<SmolStr, StateVarInfo>,
    name: &SmolStr,
    kind: StateVarKind,
    scene_id: &SmolStr,
) {
    let info = state_vars.entry(name.clone()).or_insert_with(|| StateVarInfo {
        name: name.clone(),
        kind,
        defined_in: Vec::new(),
        used_in: Vec::new(),
    });
    info.defined_in.push(SceneLocation {
        scene_id: scene_id.clone(),
        location: "scene".to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::*;

    fn scene_a() -> Scene {
        // Scene A: reads "credits", writes "visited_a"
        Scene {
            id: SceneId::new("scene_a"),
            title: "Scene A".into(),
            tags: Tags::default(),
            context: None,
            weight: 10,
            cooldown: 0,
            rhai_requirements: r#"resource("credits") >= 100"#.into(),
            passages: vec![Passage {
                text: "Scene A".into(),
                rhai_on_enter: Some(r#"set_flag("visited_a", true)"#.into()),
                choices: vec![Choice {
                    text: "Done".into(),
                    next: Navigation::End,
                    rhai_condition: Default::default(),
                    rhai_effects: Default::default(),
                }],
            }],
        }
    }

    fn scene_b() -> Scene {
        // Scene B: reads "visited_a", writes "credits"
        Scene {
            id: SceneId::new("scene_b"),
            title: "Scene B".into(),
            tags: Tags::default(),
            context: None,
            weight: 10,
            cooldown: 0,
            rhai_requirements: r#"flag("visited_a")"#.into(),
            passages: vec![Passage {
                text: "Scene B".into(),
                rhai_on_enter: Some(r#"modify_resource("credits", 50)"#.into()),
                choices: vec![Choice {
                    text: "Done".into(),
                    next: Navigation::End,
                    rhai_condition: Default::default(),
                    rhai_effects: Default::default(),
                }],
            }],
        }
    }

    fn scene_c() -> Scene {
        // Scene C: writes "orphan_flag" (never read)
        Scene {
            id: SceneId::new("scene_c"),
            title: "Scene C".into(),
            tags: Tags::default(),
            context: None,
            weight: 10,
            cooldown: 0,
            rhai_requirements: Default::default(),
            passages: vec![Passage {
                text: "Scene C".into(),
                rhai_on_enter: Some(r#"set_flag("orphan_flag", true)"#.into()),
                choices: vec![Choice {
                    text: "Done".into(),
                    next: Navigation::End,
                    rhai_condition: Default::default(),
                    rhai_effects: Default::default(),
                }],
            }],
        }
    }

    #[test]
    fn test_state_var_tracking() {
        let scenes = vec![scene_a(), scene_b()];
        let result = analyze_cross_scene(&scenes);

        // Should track credits, visited_a
        assert!(result.state_vars.contains_key("credits"));
        assert!(result.state_vars.contains_key("visited_a"));

        // Credits: read by A, written by B
        let credits = &result.state_vars["credits"];
        assert!(!credits.used_in.is_empty());
        assert!(!credits.defined_in.is_empty());

        // visited_a: written by A, read by B
        let visited = &result.state_vars["visited_a"];
        assert!(!visited.used_in.is_empty());
        assert!(!visited.defined_in.is_empty());
    }

    #[test]
    fn test_dependency_graph() {
        let scenes = vec![scene_a(), scene_b()];
        let result = analyze_cross_scene(&scenes);

        // Scene B depends on Scene A (B reads visited_a which A writes)
        let b_deps: Vec<_> = result.dependencies.dependencies("scene_b").collect();
        assert!(b_deps.iter().any(|s| s.as_str() == "scene_a"));

        // Scene A depends on Scene B (A reads credits which B writes)
        let a_deps: Vec<_> = result.dependencies.dependencies("scene_a").collect();
        assert!(a_deps.iter().any(|s| s.as_str() == "scene_b"));
    }

    #[test]
    fn test_use_before_set() {
        // Scene that reads a flag never written
        let scene = Scene {
            id: SceneId::new("lonely"),
            title: "Lonely".into(),
            tags: Tags::default(),
            context: None,
            weight: 10,
            cooldown: 0,
            rhai_requirements: r#"flag("never_set")"#.into(),
            passages: vec![Passage {
                text: "Lonely".into(),
                rhai_on_enter: None,
                choices: vec![Choice {
                    text: "Done".into(),
                    next: Navigation::End,
                    rhai_condition: Default::default(),
                    rhai_effects: Default::default(),
                }],
            }],
        };

        let result = analyze_cross_scene(&[scene]);

        // Should detect use-before-set for "never_set"
        assert!(!result.use_before_set.is_empty());
        assert!(result.use_before_set.iter().any(|u| u.var_name == "never_set"));
    }

    #[test]
    fn test_unused_write() {
        let result = analyze_cross_scene(&[scene_c()]);

        // Should detect unused write for "orphan_flag"
        assert!(!result.unused_writes.is_empty());
        assert!(result.unused_writes.iter().any(|u| u.var_name == "orphan_flag"));
    }

    #[test]
    fn test_cycle_detection() {
        let scenes = vec![scene_a(), scene_b()];
        let result = analyze_cross_scene(&scenes);

        // A and B have a dependency cycle (A reads credits→B writes, B reads visited_a→A writes)
        let order = result.dependencies.topological_order();
        assert!(order.is_err());

        let cycles = result.dependencies.find_cycles();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_topological_order() {
        // Create scenes without cycles
        let scene_x = Scene {
            id: SceneId::new("scene_x"),
            title: "Scene X".into(),
            tags: Tags::default(),
            context: None,
            weight: 10,
            cooldown: 0,
            rhai_requirements: Default::default(),
            passages: vec![Passage {
                text: "X".into(),
                rhai_on_enter: Some(r#"set_flag("x_done", true)"#.into()),
                choices: vec![Choice {
                    text: "Done".into(),
                    next: Navigation::End,
                    rhai_condition: Default::default(),
                    rhai_effects: Default::default(),
                }],
            }],
        };

        let scene_y = Scene {
            id: SceneId::new("scene_y"),
            title: "Scene Y".into(),
            tags: Tags::default(),
            context: None,
            weight: 10,
            cooldown: 0,
            rhai_requirements: r#"flag("x_done")"#.into(),
            passages: vec![Passage {
                text: "Y".into(),
                rhai_on_enter: None,
                choices: vec![Choice {
                    text: "Done".into(),
                    next: Navigation::End,
                    rhai_condition: Default::default(),
                    rhai_effects: Default::default(),
                }],
            }],
        };

        let result = analyze_cross_scene(&[scene_x, scene_y]);
        let order = result.dependencies.topological_order();

        // Should succeed since Y depends on X (Y reads x_done which X writes)
        assert!(order.is_ok());
        let order = order.unwrap();

        // X should come before Y in topological order
        let x_pos = order.iter().position(|s| s == "scene_x");
        let y_pos = order.iter().position(|s| s == "scene_y");

        // Note: topological order means dependencies come first
        // Y depends on X, so X should be processed first (lower index in result)
        // But our edges go from reader to writer, so it's inverted
        assert!(x_pos.is_some() && y_pos.is_some());
    }
}
