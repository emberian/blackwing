//! Scope management for nested game contexts.
//!
//! Scopes represent nested contexts like:
//! - Location (player is at market_square)
//!   - Conversation (talking to merchant_bob)
//!     - Scene (a specific dialogue scene)
//!
//! Each scope can have its own local state that is accessible from scripts.

use crate::Value;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// The kind of scope (what context type it represents)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScopeKind {
    /// Player is at a location
    Location,
    /// In a room within a location
    Room,
    /// In a conversation with an NPC
    Conversation,
    /// In a narrative scene
    Scene,
    /// In combat
    Combat,
    /// In a quest stage
    Quest,
    /// Custom scope kind
    Custom(SmolStr),
}

impl ScopeKind {
    pub fn as_str(&self) -> &str {
        match self {
            ScopeKind::Location => "location",
            ScopeKind::Room => "room",
            ScopeKind::Conversation => "conversation",
            ScopeKind::Scene => "scene",
            ScopeKind::Combat => "combat",
            ScopeKind::Quest => "quest",
            ScopeKind::Custom(s) => s.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "location" => ScopeKind::Location,
            "room" => ScopeKind::Room,
            "conversation" => ScopeKind::Conversation,
            "scene" => ScopeKind::Scene,
            "combat" => ScopeKind::Combat,
            "quest" => ScopeKind::Quest,
            other => ScopeKind::Custom(other.into()),
        }
    }
}

/// A single scope in the stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    /// What kind of scope this is
    pub kind: ScopeKind,
    /// The ID of this scope (e.g., location ID, scene ID)
    pub id: SmolStr,
    /// Local state for this scope (accessible from scripts)
    pub local_state: FxHashMap<SmolStr, Value>,
    /// Optional reference to an entity associated with this scope
    pub entity_ref: Option<SmolStr>,
}

impl Scope {
    pub fn new(kind: ScopeKind, id: impl Into<SmolStr>) -> Self {
        Self {
            kind,
            id: id.into(),
            local_state: FxHashMap::default(),
            entity_ref: None,
        }
    }

    pub fn with_entity(kind: ScopeKind, id: impl Into<SmolStr>, entity: impl Into<SmolStr>) -> Self {
        Self {
            kind,
            id: id.into(),
            local_state: FxHashMap::default(),
            entity_ref: Some(entity.into()),
        }
    }

    /// Get a local variable
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.local_state.get(key)
    }

    /// Set a local variable
    pub fn set(&mut self, key: impl Into<SmolStr>, value: Value) {
        self.local_state.insert(key.into(), value);
    }

    /// Remove a local variable
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.local_state.remove(key)
    }
}

/// A stack of scopes for tracking nested contexts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a new scope onto the stack
    pub fn push(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    /// Pop the top scope from the stack
    pub fn pop(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    /// Get the current (top) scope
    pub fn current(&self) -> Option<&Scope> {
        self.scopes.last()
    }

    /// Get the current scope mutably
    pub fn current_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }

    /// Get a scope at a specific depth (0 = bottom, len-1 = top)
    pub fn at(&self, depth: usize) -> Option<&Scope> {
        self.scopes.get(depth)
    }

    /// Get the depth of the stack
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }

    /// Find the nearest scope of a given kind
    pub fn find(&self, kind: &ScopeKind) -> Option<&Scope> {
        self.scopes.iter().rev().find(|s| &s.kind == kind)
    }

    /// Find the nearest scope of a given kind mutably
    pub fn find_mut(&mut self, kind: &ScopeKind) -> Option<&mut Scope> {
        self.scopes.iter_mut().rev().find(|s| &s.kind == kind)
    }

    /// Check if we're currently in a scope of the given kind
    pub fn is_in(&self, kind: &ScopeKind) -> bool {
        self.scopes.iter().any(|s| &s.kind == kind)
    }

    /// Get the ID of the current location (if any)
    pub fn current_location(&self) -> Option<&str> {
        self.find(&ScopeKind::Location).map(|s| s.id.as_str())
    }

    /// Pop all scopes until (and including) a scope of the given kind
    pub fn pop_until(&mut self, kind: &ScopeKind) -> Vec<Scope> {
        let mut popped = Vec::new();
        while let Some(scope) = self.scopes.pop() {
            let matches = &scope.kind == kind;
            popped.push(scope);
            if matches {
                break;
            }
        }
        popped
    }

    /// Iterate over all scopes from bottom to top
    pub fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.scopes.iter()
    }

    /// Iterate over all scopes from top to bottom
    pub fn iter_rev(&self) -> impl Iterator<Item = &Scope> {
        self.scopes.iter().rev()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_local_state() {
        let mut scope = Scope::new(ScopeKind::Scene, "test_scene");
        scope.set("counter", Value::Int(0));
        scope.set("name", Value::String("test".into()));

        assert_eq!(scope.get("counter").and_then(|v| v.as_int()), Some(0));
        assert_eq!(scope.get("name").and_then(|v| v.as_str()), Some("test"));
    }

    #[test]
    fn scope_stack_push_pop() {
        let mut stack = ScopeStack::new();
        assert!(stack.is_empty());

        stack.push(Scope::new(ScopeKind::Location, "market"));
        stack.push(Scope::new(ScopeKind::Conversation, "bob_dialogue"));

        assert_eq!(stack.depth(), 2);
        assert_eq!(stack.current().unwrap().id.as_str(), "bob_dialogue");

        let popped = stack.pop().unwrap();
        assert_eq!(popped.id.as_str(), "bob_dialogue");
        assert_eq!(stack.current().unwrap().id.as_str(), "market");
    }

    #[test]
    fn scope_stack_find() {
        let mut stack = ScopeStack::new();
        stack.push(Scope::new(ScopeKind::Location, "tavern"));
        stack.push(Scope::new(ScopeKind::Conversation, "innkeeper"));
        stack.push(Scope::new(ScopeKind::Scene, "quest_offer"));

        assert!(stack.is_in(&ScopeKind::Location));
        assert!(stack.is_in(&ScopeKind::Conversation));
        assert!(!stack.is_in(&ScopeKind::Combat));

        let loc = stack.find(&ScopeKind::Location).unwrap();
        assert_eq!(loc.id.as_str(), "tavern");
    }

    #[test]
    fn scope_stack_pop_until() {
        let mut stack = ScopeStack::new();
        stack.push(Scope::new(ScopeKind::Location, "dungeon"));
        stack.push(Scope::new(ScopeKind::Room, "entrance"));
        stack.push(Scope::new(ScopeKind::Combat, "goblin_fight"));

        let popped = stack.pop_until(&ScopeKind::Room);
        assert_eq!(popped.len(), 2); // Combat + Room
        assert_eq!(stack.depth(), 1);
        assert_eq!(stack.current().unwrap().kind, ScopeKind::Location);
    }
}
