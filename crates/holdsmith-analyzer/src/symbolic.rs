//! Symbolic state representation using Z3.
//!
//! This module provides symbolic execution capabilities for scene analysis:
//! - Encode game state variables (resources, flags) as Z3 symbolic variables
//! - Encode script reads as constraints (preconditions)
//! - Encode script writes as state transitions
//! - Check reachability and satisfiability of conditions

use std::collections::HashMap;

use engine_script::{AnalyzedEffect, ScriptAnalysis, StateRef};
use smol_str::SmolStr;
use z3::ast::{Ast, Bool, Int};
use z3::{Config, Context, SatResult, Solver};

/// A symbolic representation of game state.
pub struct SymbolicState<'ctx> {
    ctx: &'ctx Context,
    solver: Solver<'ctx>,
    /// Symbolic integer variables for resources
    resources: HashMap<SmolStr, Int<'ctx>>,
    /// Symbolic boolean variables for flags (simplified - real flags can be int/string)
    flags: HashMap<SmolStr, Bool<'ctx>>,
    /// Symbolic integer variables for faction reputations
    reputations: HashMap<SmolStr, Int<'ctx>>,
    /// Counter for generating unique variable names during state transitions
    version: usize,
}

impl<'ctx> SymbolicState<'ctx> {
    /// Create a new symbolic state with the given context.
    pub fn new(ctx: &'ctx Context) -> Self {
        Self {
            ctx,
            solver: Solver::new(ctx),
            resources: HashMap::new(),
            flags: HashMap::new(),
            reputations: HashMap::new(),
            version: 0,
        }
    }

    /// Get or create a symbolic variable for a resource.
    pub fn resource(&mut self, name: &str) -> Int<'ctx> {
        if let Some(var) = self.resources.get(name) {
            var.clone()
        } else {
            let var = Int::new_const(self.ctx, format!("resource_{}", name));
            self.resources.insert(SmolStr::new(name), var.clone());
            var
        }
    }

    /// Get or create a symbolic variable for a flag.
    pub fn flag(&mut self, name: &str) -> Bool<'ctx> {
        if let Some(var) = self.flags.get(name) {
            var.clone()
        } else {
            let var = Bool::new_const(self.ctx, format!("flag_{}", name));
            self.flags.insert(SmolStr::new(name), var.clone());
            var
        }
    }

    /// Get or create a symbolic variable for a faction reputation.
    pub fn reputation(&mut self, faction: &str) -> Int<'ctx> {
        if let Some(var) = self.reputations.get(faction) {
            var.clone()
        } else {
            let var = Int::new_const(self.ctx, format!("rep_{}", faction));
            self.reputations.insert(SmolStr::new(faction), var.clone());
            var
        }
    }

    /// Add constraints from script analysis reads.
    /// This encodes what the script needs to be true to execute.
    pub fn encode_reads(&mut self, analysis: &ScriptAnalysis) {
        for state_ref in &analysis.reads {
            match state_ref {
                StateRef::Resource(name) => {
                    // Just ensure the resource variable exists
                    self.resource(name);
                }
                StateRef::Flag(name) => {
                    // Just ensure the flag variable exists
                    self.flag(name);
                }
                StateRef::Tag { .. } => {
                    // Tags are typically static properties, not symbolic
                }
                StateRef::Reputation(faction) => {
                    self.reputation(faction);
                }
                StateRef::Rng => {
                    // RNG is non-deterministic, can't encode symbolically
                }
            }
        }
    }

    /// Apply effects from script analysis as state transitions.
    /// Returns the new version of the state (for SSA-style tracking).
    pub fn apply_writes(&mut self, analysis: &ScriptAnalysis) -> usize {
        self.version += 1;
        let v = self.version;

        for effect in &analysis.writes {
            match effect {
                AnalyzedEffect::Damage { resource, amount } => {
                    let old_var = self.resource(resource);
                    let new_var = Int::new_const(self.ctx, format!("resource_{}_v{}", resource, v));

                    if let Some(amt) = amount {
                        // resource' = resource - amount
                        let delta = Int::from_i64(self.ctx, *amt);
                        self.solver.assert(&new_var._eq(&(old_var - delta)));
                    }
                    // If amount is unknown, we can't constrain the new value

                    self.resources.insert(resource.clone(), new_var);
                }
                AnalyzedEffect::ModifyResource { resource, delta } => {
                    let old_var = self.resource(resource);
                    let new_var = Int::new_const(self.ctx, format!("resource_{}_v{}", resource, v));

                    if let Some(d) = delta {
                        // resource' = resource + delta
                        let delta_val = Int::from_i64(self.ctx, *d);
                        self.solver.assert(&new_var._eq(&(old_var + delta_val)));
                    }

                    self.resources.insert(resource.clone(), new_var);
                }
                AnalyzedEffect::SetResource { resource, value } => {
                    let new_var = Int::new_const(self.ctx, format!("resource_{}_v{}", resource, v));

                    if let Some(val) = value {
                        // resource' = value
                        let val_const = Int::from_i64(self.ctx, *val);
                        self.solver.assert(&new_var._eq(&val_const));
                    }

                    self.resources.insert(resource.clone(), new_var);
                }
                AnalyzedEffect::SetFlag { flag } => {
                    let new_var = Bool::new_const(self.ctx, format!("flag_{}_v{}", flag, v));
                    // Flag is set to true
                    self.solver.assert(&new_var);
                    self.flags.insert(flag.clone(), new_var);
                }
                AnalyzedEffect::ModifyReputation { faction, delta } => {
                    let old_var = self.reputation(faction);
                    let new_var = Int::new_const(self.ctx, format!("rep_{}_v{}", faction, v));

                    if let Some(d) = delta {
                        let delta_val = Int::from_i64(self.ctx, *d);
                        self.solver.assert(&new_var._eq(&(old_var + delta_val)));
                    }

                    self.reputations.insert(faction.clone(), new_var);
                }
                // Effects that don't directly modify tracked state
                AnalyzedEffect::AddCard { .. }
                | AnalyzedEffect::RemoveCards { .. }
                | AnalyzedEffect::Chronicle { .. }
                | AnalyzedEffect::JumpTo { .. } => {}
            }
        }

        v
    }

    /// Add a constraint that a resource must be >= some value.
    pub fn assert_resource_ge(&mut self, name: &str, value: i64) {
        let var = self.resource(name);
        let val = Int::from_i64(self.ctx, value);
        self.solver.assert(&var.ge(&val));
    }

    /// Add a constraint that a resource must be < some value.
    pub fn assert_resource_lt(&mut self, name: &str, value: i64) {
        let var = self.resource(name);
        let val = Int::from_i64(self.ctx, value);
        self.solver.assert(&var.lt(&val));
    }

    /// Add a constraint that a flag must be true.
    pub fn assert_flag_true(&mut self, name: &str) {
        let var = self.flag(name);
        self.solver.assert(&var);
    }

    /// Add a constraint that a flag must be false.
    pub fn assert_flag_false(&mut self, name: &str) {
        let var = self.flag(name);
        self.solver.assert(&var.not());
    }

    /// Add an arbitrary boolean constraint.
    pub fn assert_constraint(&mut self, constraint: &Bool<'ctx>) {
        self.solver.assert(constraint);
    }

    /// Check if the current constraints are satisfiable.
    pub fn check(&self) -> SatResult {
        self.solver.check()
    }

    /// Check if a given condition is satisfiable under current constraints.
    pub fn check_with(&self, additional: &Bool<'ctx>) -> SatResult {
        self.solver.push();
        self.solver.assert(additional);
        let result = self.solver.check();
        self.solver.pop(1);
        result
    }

    /// Check if a condition is always true (valid) under current constraints.
    /// Returns true if the negation is unsatisfiable.
    pub fn is_valid(&self, condition: &Bool<'ctx>) -> bool {
        self.check_with(&condition.not()) == SatResult::Unsat
    }

    /// Check if a condition is always false (unsatisfiable) under current constraints.
    pub fn is_unsat(&self, condition: &Bool<'ctx>) -> bool {
        self.check_with(condition) == SatResult::Unsat
    }

    /// Push a new scope for backtracking.
    pub fn push(&self) {
        self.solver.push();
    }

    /// Pop a scope, undoing assertions since the last push.
    pub fn pop(&self) {
        self.solver.pop(1);
    }

    /// Get the Z3 context.
    pub fn context(&self) -> &'ctx Context {
        self.ctx
    }

    /// Create a constraint: resource >= value
    pub fn resource_ge(&mut self, name: &str, value: i64) -> Bool<'ctx> {
        let var = self.resource(name);
        let val = Int::from_i64(self.ctx, value);
        var.ge(&val)
    }

    /// Create a constraint: resource < value
    pub fn resource_lt(&mut self, name: &str, value: i64) -> Bool<'ctx> {
        let var = self.resource(name);
        let val = Int::from_i64(self.ctx, value);
        var.lt(&val)
    }

    /// Create a constraint: resource == value
    pub fn resource_eq(&mut self, name: &str, value: i64) -> Bool<'ctx> {
        let var = self.resource(name);
        let val = Int::from_i64(self.ctx, value);
        var._eq(&val)
    }

    /// Add a constraint that a resource must equal some value.
    pub fn assert_resource_eq(&mut self, name: &str, value: i64) {
        let constraint = self.resource_eq(name, value);
        self.solver.assert(&constraint);
    }

    /// Get a model satisfying current constraints (if SAT).
    /// Returns None if constraints are unsatisfiable.
    pub fn get_model(&self) -> Option<z3::Model<'ctx>> {
        if self.solver.check() == SatResult::Sat {
            self.solver.get_model()
        } else {
            None
        }
    }

    /// Extract concrete values from a model for all tracked variables.
    /// Returns (resources, flags) where each is a vec of (name, value).
    pub fn extract_concrete_state(&self, model: &z3::Model<'ctx>) -> (Vec<(SmolStr, i64)>, Vec<(SmolStr, bool)>) {
        let mut resources = Vec::new();
        let mut flags = Vec::new();

        for (name, var) in &self.resources {
            if let Some(val) = model.eval(var, true) {
                if let Some(i) = val.as_i64() {
                    resources.push((name.clone(), i));
                }
            }
        }

        for (name, var) in &self.flags {
            if let Some(val) = model.eval(var, true) {
                if let Some(b) = val.as_bool() {
                    flags.push((name.clone(), b));
                }
            }
        }

        // Sort for consistent output
        resources.sort_by(|a, b| a.0.cmp(&b.0));
        flags.sort_by(|a, b| a.0.cmp(&b.0));

        (resources, flags)
    }

    /// Get the names of all tracked resources.
    pub fn tracked_resources(&self) -> impl Iterator<Item = &SmolStr> {
        self.resources.keys()
    }

    /// Get the names of all tracked flags.
    pub fn tracked_flags(&self) -> impl Iterator<Item = &SmolStr> {
        self.flags.keys()
    }
}

/// Create a Z3 context with default configuration.
pub fn create_context() -> Context {
    let config = Config::new();
    Context::new(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_constraints() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);

        // credits >= 100
        state.assert_resource_ge("credits", 100);

        // Should be satisfiable
        assert_eq!(state.check(), SatResult::Sat);

        // Check if credits >= 200 is possible
        let ge_200 = state.resource_ge("credits", 200);
        assert_eq!(state.check_with(&ge_200), SatResult::Sat);

        // Add constraint credits < 150
        state.assert_resource_lt("credits", 150);

        // Now credits >= 200 should be unsat
        assert_eq!(state.check_with(&ge_200), SatResult::Unsat);

        // But credits >= 100 && credits < 150 is still sat
        assert_eq!(state.check(), SatResult::Sat);
    }

    #[test]
    fn test_apply_effects() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);

        // Start with credits = 100
        state.assert_resource_eq("credits", 100);

        // Apply effect: modify_resource("credits", -50)
        let mut analysis = ScriptAnalysis::default();
        analysis.writes.push(AnalyzedEffect::ModifyResource {
            resource: SmolStr::new("credits"),
            delta: Some(-50),
        });

        state.apply_writes(&analysis);

        // After the effect, credits should be 50
        let eq_50 = state.resource_eq("credits", 50);
        assert!(state.is_valid(&eq_50));
    }

    #[test]
    fn test_flag_constraints() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);

        // Initially unconstrained
        assert_eq!(state.check(), SatResult::Sat);

        // Set flag to true
        let mut analysis = ScriptAnalysis::default();
        analysis.writes.push(AnalyzedEffect::SetFlag {
            flag: SmolStr::new("visited"),
        });

        state.apply_writes(&analysis);

        // Flag should be true
        let flag_var = state.flag("visited");
        assert!(state.is_valid(&flag_var));
    }

    #[test]
    fn test_push_pop() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);

        state.assert_resource_ge("credits", 100);
        assert_eq!(state.check(), SatResult::Sat);

        state.push();
        state.assert_resource_lt("credits", 50); // Contradicts >= 100
        assert_eq!(state.check(), SatResult::Unsat);

        state.pop();
        // Back to just >= 100, should be sat again
        assert_eq!(state.check(), SatResult::Sat);
    }
}
