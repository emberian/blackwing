//! Entity storage and management.
//!
//! Entities are the core building blocks of the game world. Each entity has:
//! - A unique key (internal slot map key)
//! - An EntityId (kind + id for external reference)
//! - Tags for classification
//! - Components (dynamic key-value data)

use engine_primitives::{EntityId, Tags, TagsExt, Value};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};
use smol_str::SmolStr;

new_key_type! {
    /// Internal key for entity storage
    pub struct EntityKey;
}

/// An entity in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub tags: Tags,
}

impl Entity {
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            tags: Tags::new(),
        }
    }

    pub fn with_tags(id: EntityId, tags: Tags) -> Self {
        Self { id, tags }
    }
}

/// Storage for all entities and their components
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityStorage {
    /// Primary entity storage
    entities: SlotMap<EntityKey, Entity>,
    /// Component data: entity key -> component name -> value
    components: FxHashMap<EntityKey, FxHashMap<SmolStr, Value>>,
    /// Index: EntityId -> EntityKey (for lookups by ID)
    id_to_key: FxHashMap<String, EntityKey>,
    /// Index: kind -> list of entity keys (for queries by kind)
    by_kind: FxHashMap<SmolStr, Vec<EntityKey>>,
}

impl EntityStorage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a new entity with the given ID
    pub fn spawn(&mut self, id: EntityId) -> EntityKey {
        let entity = Entity::new(id.clone());
        let key = self.entities.insert(entity);

        // Update indices
        self.id_to_key.insert(id.as_qualified(), key);
        self.by_kind
            .entry(id.kind.clone())
            .or_default()
            .push(key);

        key
    }

    /// Spawn a new entity with tags
    pub fn spawn_with_tags(&mut self, id: EntityId, tags: Tags) -> EntityKey {
        let entity = Entity::with_tags(id.clone(), tags);
        let key = self.entities.insert(entity);

        self.id_to_key.insert(id.as_qualified(), key);
        self.by_kind
            .entry(id.kind.clone())
            .or_default()
            .push(key);

        key
    }

    /// Despawn an entity by key
    pub fn despawn(&mut self, key: EntityKey) -> Option<Entity> {
        let entity = self.entities.remove(key)?;

        // Clean up indices
        self.id_to_key.remove(&entity.id.as_qualified());
        if let Some(keys) = self.by_kind.get_mut(&entity.id.kind) {
            keys.retain(|&k| k != key);
        }
        self.components.remove(&key);

        Some(entity)
    }

    /// Despawn an entity by ID
    pub fn despawn_by_id(&mut self, id: &EntityId) -> Option<Entity> {
        let key = self.id_to_key.get(&id.as_qualified()).copied()?;
        self.despawn(key)
    }

    /// Get an entity by key
    pub fn get(&self, key: EntityKey) -> Option<&Entity> {
        self.entities.get(key)
    }

    /// Get an entity mutably by key
    pub fn get_mut(&mut self, key: EntityKey) -> Option<&mut Entity> {
        self.entities.get_mut(key)
    }

    /// Look up an entity key by ID
    pub fn key_of(&self, id: &EntityId) -> Option<EntityKey> {
        self.id_to_key.get(&id.as_qualified()).copied()
    }

    /// Get an entity by ID
    pub fn get_by_id(&self, id: &EntityId) -> Option<&Entity> {
        let key = self.key_of(id)?;
        self.get(key)
    }

    /// Get an entity mutably by ID
    pub fn get_by_id_mut(&mut self, id: &EntityId) -> Option<&mut Entity> {
        let key = self.key_of(id)?;
        self.get_mut(key)
    }

    /// Check if an entity exists
    pub fn contains(&self, key: EntityKey) -> bool {
        self.entities.contains_key(key)
    }

    /// Check if an entity exists by ID
    pub fn contains_id(&self, id: &EntityId) -> bool {
        self.id_to_key.contains_key(&id.as_qualified())
    }

    /// Get all entity keys of a given kind
    pub fn keys_of_kind(&self, kind: &str) -> impl Iterator<Item = EntityKey> + '_ {
        self.by_kind
            .get(kind)
            .map(|v| v.iter().copied())
            .into_iter()
            .flatten()
    }

    /// Get all entities of a given kind
    pub fn entities_of_kind(&self, kind: &str) -> impl Iterator<Item = (EntityKey, &Entity)> + '_ {
        self.keys_of_kind(kind)
            .filter_map(|key| self.entities.get(key).map(|e| (key, e)))
    }

    /// Iterate over all entities
    pub fn iter(&self) -> impl Iterator<Item = (EntityKey, &Entity)> + '_ {
        self.entities.iter()
    }

    /// Get the number of entities
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Check if the storage is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    // === Component Operations ===

    /// Set a component value on an entity
    pub fn set_component(&mut self, key: EntityKey, component: impl Into<SmolStr>, value: Value) {
        self.components
            .entry(key)
            .or_default()
            .insert(component.into(), value);
    }

    /// Get a component value from an entity
    pub fn get_component(&self, key: EntityKey, component: &str) -> Option<&Value> {
        self.components.get(&key)?.get(component)
    }

    /// Get a component value mutably
    pub fn get_component_mut(&mut self, key: EntityKey, component: &str) -> Option<&mut Value> {
        self.components.get_mut(&key)?.get_mut(component)
    }

    /// Remove a component from an entity
    pub fn remove_component(&mut self, key: EntityKey, component: &str) -> Option<Value> {
        self.components.get_mut(&key)?.remove(component)
    }

    /// Check if an entity has a component
    pub fn has_component(&self, key: EntityKey, component: &str) -> bool {
        self.components
            .get(&key)
            .map(|c| c.contains_key(component))
            .unwrap_or(false)
    }

    /// Get all components for an entity
    pub fn components(&self, key: EntityKey) -> Option<&FxHashMap<SmolStr, Value>> {
        self.components.get(&key)
    }

    /// Get all components for an entity mutably
    pub fn components_mut(&mut self, key: EntityKey) -> Option<&mut FxHashMap<SmolStr, Value>> {
        self.components.get_mut(&key)
    }

    // === Tag Operations ===

    /// Add a tag to an entity
    pub fn add_tag(&mut self, key: EntityKey, tag: impl Into<SmolStr>) {
        if let Some(entity) = self.entities.get_mut(key) {
            entity.tags.add_tag(tag.into());
        }
    }

    /// Remove a tag from an entity
    pub fn remove_tag(&mut self, key: EntityKey, tag: &str) -> bool {
        self.entities
            .get_mut(key)
            .map(|e| e.tags.remove_tag(tag))
            .unwrap_or(false)
    }

    /// Check if an entity has a tag
    pub fn has_tag(&self, key: EntityKey, tag: &str) -> bool {
        self.entities
            .get(key)
            .map(|e| e.tags.has_tag(tag))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_and_get() {
        let mut storage = EntityStorage::new();
        let id = EntityId::new("npc", "bob");
        let key = storage.spawn(id.clone());

        assert!(storage.contains(key));
        assert!(storage.contains_id(&id));

        let entity = storage.get(key).unwrap();
        assert_eq!(entity.id, id);
    }

    #[test]
    fn despawn() {
        let mut storage = EntityStorage::new();
        let id = EntityId::new("item", "sword");
        let key = storage.spawn(id.clone());

        storage.set_component(key, "damage", Value::Int(10));

        let entity = storage.despawn(key).unwrap();
        assert_eq!(entity.id, id);
        assert!(!storage.contains(key));
        assert!(!storage.contains_id(&id));
        assert!(storage.get_component(key, "damage").is_none());
    }

    #[test]
    fn components() {
        let mut storage = EntityStorage::new();
        let key = storage.spawn(EntityId::new("npc", "alice"));

        storage.set_component(key, "health", Value::Int(100));
        storage.set_component(key, "name", Value::String("Alice".into()));

        assert_eq!(
            storage.get_component(key, "health").and_then(|v| v.as_int()),
            Some(100)
        );
        assert_eq!(
            storage.get_component(key, "name").and_then(|v| v.as_str()),
            Some("Alice")
        );
        assert!(storage.get_component(key, "mana").is_none());
    }

    #[test]
    fn tags() {
        let mut storage = EntityStorage::new();
        let key = storage.spawn(EntityId::new("npc", "guard"));

        storage.add_tag(key, "hostile");
        storage.add_tag(key, "armed");

        assert!(storage.has_tag(key, "hostile"));
        assert!(storage.has_tag(key, "armed"));
        assert!(!storage.has_tag(key, "friendly"));

        storage.remove_tag(key, "hostile");
        assert!(!storage.has_tag(key, "hostile"));
    }

    #[test]
    fn query_by_kind() {
        let mut storage = EntityStorage::new();
        storage.spawn(EntityId::new("npc", "bob"));
        storage.spawn(EntityId::new("npc", "alice"));
        storage.spawn(EntityId::new("item", "sword"));

        let npcs: Vec<_> = storage.entities_of_kind("npc").collect();
        assert_eq!(npcs.len(), 2);

        let items: Vec<_> = storage.entities_of_kind("item").collect();
        assert_eq!(items.len(), 1);
    }
}
