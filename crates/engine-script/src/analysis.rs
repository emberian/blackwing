//! Script analysis for extracting reads, writes, and control flow from Rhai scripts.
//!
//! This module provides static analysis of Rhai scripts to understand:
//! - What game state they read (resources, flags, tags, reputation)
//! - What game state they write (effects)
//! - Control flow (branches, RNG usage, navigation)
//!
//! Uses Rhai's `AST::walk()` API for stable AST inspection.

use std::collections::HashSet;

use rhai::{ASTNode, Engine, Expr, Stmt, AST};
use smol_str::SmolStr;

use crate::ScriptError;

/// A reference to game state that can be read or written.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StateRef {
    /// A resource like "credits", "hull", "fuel"
    Resource(SmolStr),
    /// A flag like "sera_survived"
    Flag(SmolStr),
    /// A tag check like ("ship", "combat")
    Tag { category: SmolStr, tag: SmolStr },
    /// A faction reputation like "merchants"
    Reputation(SmolStr),
    /// RNG state (reading this means non-deterministic)
    Rng,
}

/// Effect that a script can produce.
#[derive(Debug, Clone, PartialEq)]
pub enum AnalyzedEffect {
    /// Damage to a resource
    Damage { resource: SmolStr, amount: Option<i64> },
    /// Modify a resource by delta
    ModifyResource { resource: SmolStr, delta: Option<i64> },
    /// Set a resource to a value
    SetResource { resource: SmolStr, value: Option<i64> },
    /// Set a flag
    SetFlag { flag: SmolStr },
    /// Add a card
    AddCard { card_id: SmolStr },
    /// Remove cards matching pattern
    RemoveCards { pattern: SmolStr },
    /// Add chronicle entry
    Chronicle { title: SmolStr },
    /// Modify faction reputation
    ModifyReputation { faction: SmolStr, delta: Option<i64> },
    /// Navigation to another passage
    JumpTo { target: SmolStr },
}

/// A branch in script control flow.
#[derive(Debug, Clone)]
pub struct Branch {
    /// Human-readable description of the condition
    pub condition_desc: String,
    /// Effects in this branch
    pub effects: Vec<AnalyzedEffect>,
    /// Is this branch deterministic (no RNG)?
    pub is_deterministic: bool,
}

/// Result of analyzing a script.
#[derive(Debug, Clone, Default)]
pub struct ScriptAnalysis {
    /// State references that are read
    pub reads: HashSet<StateRef>,
    /// Effects that are produced
    pub writes: Vec<AnalyzedEffect>,
    /// Whether the script uses RNG (non-deterministic)
    pub uses_rng: bool,
    /// Branches in control flow
    pub branches: Vec<Branch>,
    /// Navigation targets (jump_to calls)
    pub navigation_targets: Vec<SmolStr>,
}

impl ScriptAnalysis {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if this script reads a specific resource.
    pub fn reads_resource(&self, name: &str) -> bool {
        self.reads.contains(&StateRef::Resource(SmolStr::new(name)))
    }

    /// Check if this script modifies a specific resource.
    pub fn writes_resource(&self, name: &str) -> bool {
        self.writes.iter().any(|e| match e {
            AnalyzedEffect::Damage { resource, .. }
            | AnalyzedEffect::ModifyResource { resource, .. }
            | AnalyzedEffect::SetResource { resource, .. } => resource.as_str() == name,
            _ => false,
        })
    }

    /// Check if this script is deterministic (no RNG).
    pub fn is_deterministic(&self) -> bool {
        !self.uses_rng
    }
}

/// Analyze a Rhai script source string.
pub fn analyze_script(source: &str) -> Result<ScriptAnalysis, ScriptError> {
    let engine = Engine::new();
    let ast = engine.compile(source)?;
    analyze_ast(&ast)
}

/// Analyze a compiled Rhai AST using `AST::walk()`.
pub fn analyze_ast(ast: &AST) -> Result<ScriptAnalysis, ScriptError> {
    let mut analysis = ScriptAnalysis::new();

    // Use Rhai's stable AST walking API
    ast.walk(&mut |nodes: &[ASTNode]| {
        // The last element in the path is the current node
        if let Some(node) = nodes.last() {
            match node {
                // Function call as expression (e.g., `resource("credits")`)
                ASTNode::Expr(Expr::FnCall(call, _)) => {
                    analyze_fn_call(call, &mut analysis);
                }
                // Function call as statement (e.g., `damage("hull", 10);`)
                ASTNode::Stmt(Stmt::FnCall(call, _)) => {
                    analyze_fn_call(call, &mut analysis);
                }
                _ => {}
            }
        }
        true // Continue walking
    });

    Ok(analysis)
}

/// Analyze a function call to detect reads and writes.
fn analyze_fn_call(call: &rhai::FnCallExpr, analysis: &mut ScriptAnalysis) {
    let fn_name = call.name.as_str();

    // Classify the function call
    match fn_name {
        // Read functions
        "resource" => {
            if let Some(name) = extract_string_arg(&call.args, 0) {
                analysis.reads.insert(StateRef::Resource(name));
            }
        }
        "has_tag" => {
            if let (Some(cat), Some(tag)) = (extract_string_arg(&call.args, 0), extract_string_arg(&call.args, 1)) {
                analysis.reads.insert(StateRef::Tag { category: cat, tag });
            }
        }
        "flag" | "flag_bool" => {
            if let Some(name) = extract_string_arg(&call.args, 0) {
                analysis.reads.insert(StateRef::Flag(name));
            }
        }
        "reputation" => {
            if let Some(faction) = extract_string_arg(&call.args, 0) {
                analysis.reads.insert(StateRef::Reputation(faction));
            }
        }
        "rng_float" | "rng_int" => {
            analysis.reads.insert(StateRef::Rng);
            analysis.uses_rng = true;
        }

        // Write functions
        "damage" => {
            if let Some(resource) = extract_string_arg(&call.args, 0) {
                let amount = extract_int_arg(&call.args, 1);
                analysis.writes.push(AnalyzedEffect::Damage { resource, amount });
            }
        }
        "modify_resource" => {
            if let Some(resource) = extract_string_arg(&call.args, 0) {
                let delta = extract_int_arg(&call.args, 1);
                analysis.writes.push(AnalyzedEffect::ModifyResource { resource, delta });
            }
        }
        "set_resource" => {
            if let Some(resource) = extract_string_arg(&call.args, 0) {
                let value = extract_int_arg(&call.args, 1);
                analysis.writes.push(AnalyzedEffect::SetResource { resource, value });
            }
        }
        "set_flag" | "set_flag_int" | "set_flag_str" => {
            if let Some(flag) = extract_string_arg(&call.args, 0) {
                analysis.writes.push(AnalyzedEffect::SetFlag { flag });
            }
        }
        "add_card" => {
            if let Some(card_id) = extract_string_arg(&call.args, 0) {
                analysis.writes.push(AnalyzedEffect::AddCard { card_id });
            }
        }
        "remove_cards" => {
            if let Some(pattern) = extract_string_arg(&call.args, 0) {
                analysis.writes.push(AnalyzedEffect::RemoveCards { pattern });
            }
        }
        "chronicle" => {
            if let Some(title) = extract_string_arg(&call.args, 0) {
                analysis.writes.push(AnalyzedEffect::Chronicle { title });
            }
        }
        "modify_reputation" => {
            if let Some(faction) = extract_string_arg(&call.args, 0) {
                let delta = extract_int_arg(&call.args, 1);
                analysis.writes.push(AnalyzedEffect::ModifyReputation { faction, delta });
            }
        }
        "jump_to" => {
            if let Some(target) = extract_string_arg(&call.args, 0) {
                analysis.writes.push(AnalyzedEffect::JumpTo { target: target.clone() });
                analysis.navigation_targets.push(target);
            }
        }

        _ => {
            // Unknown function - could be a method call on a value
        }
    }
}

/// Try to extract a string constant from an argument.
fn extract_string_arg(args: &[Expr], index: usize) -> Option<SmolStr> {
    args.get(index).and_then(|expr| {
        if let Expr::StringConstant(s, _) = expr {
            Some(SmolStr::new(s.as_str()))
        } else {
            None
        }
    })
}

/// Try to extract an integer constant from an argument.
fn extract_int_arg(args: &[Expr], index: usize) -> Option<i64> {
    args.get(index).and_then(|expr| {
        if let Expr::IntegerConstant(n, _) = expr {
            Some(*n)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_simple_resource_read() {
        let analysis = analyze_script(r#"resource("credits") >= 100"#).unwrap();
        assert!(analysis.reads_resource("credits"));
        assert!(!analysis.uses_rng);
    }

    #[test]
    fn analyze_damage_effect() {
        let analysis = analyze_script(r#"damage("hull", 10);"#).unwrap();
        assert!(analysis.writes_resource("hull"));
        assert_eq!(analysis.writes.len(), 1);
        match &analysis.writes[0] {
            AnalyzedEffect::Damage { resource, amount } => {
                assert_eq!(resource.as_str(), "hull");
                assert_eq!(*amount, Some(10));
            }
            _ => panic!("Expected Damage effect"),
        }
    }

    #[test]
    fn analyze_rng_usage() {
        let analysis = analyze_script(r#"let x = rng_float();"#).unwrap();
        assert!(analysis.uses_rng);
        assert!(!analysis.is_deterministic());
        assert!(analysis.reads.contains(&StateRef::Rng));
    }

    #[test]
    fn analyze_tag_check() {
        let analysis = analyze_script(r#"has_tag("ship", "combat")"#).unwrap();
        assert!(analysis.reads.contains(&StateRef::Tag {
            category: SmolStr::new("ship"),
            tag: SmolStr::new("combat"),
        }));
    }

    #[test]
    fn analyze_complex_script() {
        let analysis = analyze_script(r#"
            if resource("credits") >= 100 && has_tag("ship", "combat") {
                damage("hull", 10);
                modify_resource("credits", -50);
            } else {
                set_flag("failed", true);
            }
        "#).unwrap();

        assert!(analysis.reads_resource("credits"));
        assert!(analysis.reads.contains(&StateRef::Tag {
            category: SmolStr::new("ship"),
            tag: SmolStr::new("combat"),
        }));
        assert!(analysis.writes_resource("hull"));
        assert!(analysis.writes_resource("credits"));
        assert!(analysis.writes.iter().any(|e| matches!(e, AnalyzedEffect::SetFlag { flag } if flag == "failed")));
    }

    #[test]
    fn analyze_navigation() {
        let analysis = analyze_script(r#"jump_to("next_passage");"#).unwrap();
        assert!(analysis.navigation_targets.contains(&SmolStr::new("next_passage")));
    }

    #[test]
    fn analyze_probabilistic_script() {
        let analysis = analyze_script(r#"
            let roll = rng_float();
            if roll < 0.5 {
                damage("hull", 20);
            } else {
                add_card("treasure");
            }
        "#).unwrap();

        assert!(analysis.uses_rng);
        assert!(analysis.writes_resource("hull"));
        assert!(analysis.writes.iter().any(|e| matches!(e, AnalyzedEffect::AddCard { card_id } if card_id == "treasure")));
    }
}
