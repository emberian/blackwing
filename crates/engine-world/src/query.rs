//! Entity query system.
//!
//! Provides a builder-style API for querying entities by:
//! - Kind (npc, item, location, etc.)
//! - Tags (hostile, friendly, etc.)
//! - Components (has health, has inventory, etc.)
//! - Component values (health > 50, etc.)

use engine_primitives::{EntityId, Value};
use smol_str::SmolStr;

use crate::{Entity, EntityKey, EntityStorage};

/// A query builder for finding entities
pub struct EntityQuery<'a> {
    storage: &'a EntityStorage,
    kind_filter: Option<SmolStr>,
    tag_filters: Vec<SmolStr>,
    component_filters: Vec<SmolStr>,
    predicates: Vec<Box<dyn Fn(&Entity, &EntityStorage) -> bool + 'a>>,
}

impl<'a> EntityQuery<'a> {
    /// Create a new query over the given storage
    pub fn new(storage: &'a EntityStorage) -> Self {
        Self {
            storage,
            kind_filter: None,
            tag_filters: Vec::new(),
            component_filters: Vec::new(),
            predicates: Vec::new(),
        }
    }

    /// Filter by entity kind (npc, item, location, etc.)
    pub fn kind(mut self, kind: impl Into<SmolStr>) -> Self {
        self.kind_filter = Some(kind.into());
        self
    }

    /// Filter by tag
    pub fn with_tag(mut self, tag: impl Into<SmolStr>) -> Self {
        self.tag_filters.push(tag.into());
        self
    }

    /// Filter by having a component
    pub fn with_component(mut self, component: impl Into<SmolStr>) -> Self {
        self.component_filters.push(component.into());
        self
    }

    /// Filter by a custom predicate
    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&Entity, &EntityStorage) -> bool + 'a,
    {
        self.predicates.push(Box::new(predicate));
        self
    }

    /// Execute the query and return matching entity keys
    pub fn keys(self) -> Vec<EntityKey> {
        let iter: Box<dyn Iterator<Item = (EntityKey, &Entity)>> = if let Some(kind) = &self.kind_filter {
            Box::new(self.storage.entities_of_kind(kind))
        } else {
            Box::new(self.storage.iter())
        };

        iter.filter(|(key, entity)| {
            // Check tag filters
            for tag in &self.tag_filters {
                if !self.storage.has_tag(*key, tag) {
                    return false;
                }
            }

            // Check component filters
            for component in &self.component_filters {
                if !self.storage.has_component(*key, component) {
                    return false;
                }
            }

            // Check custom predicates
            for predicate in &self.predicates {
                if !predicate(entity, self.storage) {
                    return false;
                }
            }

            true
        })
        .map(|(key, _)| key)
        .collect()
    }

    /// Execute the query and return matching entity IDs
    pub fn ids(self) -> Vec<EntityId> {
        let iter: Box<dyn Iterator<Item = (EntityKey, &Entity)>> = if let Some(kind) = &self.kind_filter {
            Box::new(self.storage.entities_of_kind(kind))
        } else {
            Box::new(self.storage.iter())
        };

        iter.filter(|(key, entity)| {
            for tag in &self.tag_filters {
                if !self.storage.has_tag(*key, tag) {
                    return false;
                }
            }

            for component in &self.component_filters {
                if !self.storage.has_component(*key, component) {
                    return false;
                }
            }

            for predicate in &self.predicates {
                if !predicate(entity, self.storage) {
                    return false;
                }
            }

            true
        })
        .map(|(_, entity)| entity.id.clone())
        .collect()
    }

    /// Count matching entities without collecting them
    pub fn count(self) -> usize {
        self.keys().len()
    }

    /// Check if any entities match
    pub fn exists(self) -> bool {
        // Could optimize this to short-circuit
        !self.keys().is_empty()
    }

    /// Get the first matching entity key
    pub fn first(self) -> Option<EntityKey> {
        self.keys().into_iter().next()
    }
}

/// Extension trait for EntityStorage to enable query building
pub trait QueryExt {
    fn query(&self) -> EntityQuery<'_>;
}

impl QueryExt for EntityStorage {
    fn query(&self) -> EntityQuery<'_> {
        EntityQuery::new(self)
    }
}

/// Helper to create component value predicates
pub fn component_eq(component: &'static str, value: Value) -> impl Fn(&Entity, &EntityStorage) -> bool {
    move |entity, storage| {
        storage
            .key_of(&entity.id)
            .and_then(|key| storage.get_component(key, component))
            .map(|v| *v == value)
            .unwrap_or(false)
    }
}

/// Helper to create numeric component comparison predicates
pub fn component_gt(component: &'static str, threshold: i64) -> impl Fn(&Entity, &EntityStorage) -> bool {
    move |entity, storage| {
        storage
            .key_of(&entity.id)
            .and_then(|key| storage.get_component(key, component))
            .and_then(|v| v.as_int())
            .map(|v| v > threshold)
            .unwrap_or(false)
    }
}

pub fn component_lt(component: &'static str, threshold: i64) -> impl Fn(&Entity, &EntityStorage) -> bool {
    move |entity, storage| {
        storage
            .key_of(&entity.id)
            .and_then(|key| storage.get_component(key, component))
            .and_then(|v| v.as_int())
            .map(|v| v < threshold)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_by_kind() {
        let mut storage = EntityStorage::new();
        storage.spawn(EntityId::new("npc", "bob"));
        storage.spawn(EntityId::new("npc", "alice"));
        storage.spawn(EntityId::new("item", "sword"));

        let npcs = storage.query().kind("npc").keys();
        assert_eq!(npcs.len(), 2);

        let items = storage.query().kind("item").keys();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn query_by_tag() {
        let mut storage = EntityStorage::new();
        let bob = storage.spawn(EntityId::new("npc", "bob"));
        let alice = storage.spawn(EntityId::new("npc", "alice"));

        storage.add_tag(bob, "hostile");
        storage.add_tag(alice, "friendly");

        let hostile = storage.query().kind("npc").with_tag("hostile").keys();
        assert_eq!(hostile.len(), 1);

        let friendly = storage.query().with_tag("friendly").keys();
        assert_eq!(friendly.len(), 1);
    }

    #[test]
    fn query_by_component() {
        let mut storage = EntityStorage::new();
        let bob = storage.spawn(EntityId::new("npc", "bob"));
        let sword = storage.spawn(EntityId::new("item", "sword"));

        storage.set_component(bob, "health", Value::Int(100));
        storage.set_component(sword, "damage", Value::Int(10));

        let with_health = storage.query().with_component("health").keys();
        assert_eq!(with_health.len(), 1);

        let with_damage = storage.query().with_component("damage").keys();
        assert_eq!(with_damage.len(), 1);
    }

    #[test]
    fn query_with_predicate() {
        let mut storage = EntityStorage::new();
        let bob = storage.spawn(EntityId::new("npc", "bob"));
        let alice = storage.spawn(EntityId::new("npc", "alice"));

        storage.set_component(bob, "health", Value::Int(100));
        storage.set_component(alice, "health", Value::Int(30));

        let healthy = storage
            .query()
            .kind("npc")
            .with_component("health")
            .filter(component_gt("health", 50))
            .keys();
        assert_eq!(healthy.len(), 1);
    }
}
