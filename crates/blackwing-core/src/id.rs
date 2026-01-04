//! Entity and resource identifiers for the game engine.
//!
//! The core identifier type is `EntityId`, which combines a `kind` (like "npc", "item", "location")
//! with a unique `id` within that kind. This replaces the old system of separate typed IDs.

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A generalized entity identifier combining kind and id.
///
/// Examples:
/// - `EntityId { kind: "npc", id: "merchant_bob" }`
/// - `EntityId { kind: "location", id: "market_square" }`
/// - `EntityId { kind: "item", id: "health_potion" }`
/// - `EntityId { kind: "actor", id: "player_1" }`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    pub kind: SmolStr,
    pub id: SmolStr,
}

impl EntityId {
    pub fn new(kind: impl Into<SmolStr>, id: impl Into<SmolStr>) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
        }
    }

    /// Parse from "kind:id" format
    pub fn parse(s: &str) -> Option<Self> {
        let (kind, id) = s.split_once(':')?;
        Some(Self::new(kind, id))
    }

    /// Format as "kind:id"
    pub fn as_qualified(&self) -> String {
        format!("{}:{}", self.kind, self.id)
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.kind, self.id)
    }
}

impl From<&str> for EntityId {
    fn from(s: &str) -> Self {
        EntityId::parse(s).unwrap_or_else(|| EntityId::new("unknown", s))
    }
}

/// A simple string-based identifier (for resources, flags, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SimpleId(SmolStr);

impl SimpleId {
    pub fn new(s: impl Into<SmolStr>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SimpleId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for SimpleId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<SmolStr> for SimpleId {
    fn from(s: SmolStr) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for SimpleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for SimpleId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// Type aliases for semantic clarity
pub type ResourceId = SimpleId;
pub type FlagId = SimpleId;
pub type SceneId = SimpleId;
pub type DialogueId = SimpleId;
pub type QuestId = SimpleId;
pub type TagId = SimpleId;
pub type TagCategoryId = SimpleId;
pub type ScopeKindId = SimpleId;

/// Actors are entities - this is just a type alias for clarity
pub type ActorId = EntityId;

/// Trait for objects that can provide tag information.
pub trait TagProvider {
    fn has_tag(&self, category: &TagCategoryId, tag: &TagId) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_id_parse() {
        let id = EntityId::parse("npc:merchant_bob").unwrap();
        assert_eq!(id.kind(), "npc");
        assert_eq!(id.id(), "merchant_bob");
        assert_eq!(id.as_qualified(), "npc:merchant_bob");
    }

    #[test]
    fn entity_id_display() {
        let id = EntityId::new("item", "sword");
        assert_eq!(format!("{}", id), "item:sword");
    }
}
