use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagId(SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagCategoryId(SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardTypeId(SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardId(SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardInstanceId(SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SceneId(SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationId(SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactionId(SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlagId(SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextId(SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SlotId(SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChronicleEntryId(SmolStr);

macro_rules! impl_id {
    ($ty:ty) => {
        impl $ty {
            pub fn new(s: impl Into<SmolStr>) -> Self {
                Self(s.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $ty {
            fn from(s: &str) -> Self {
                Self::new(s)
            }
        }

        impl From<String> for $ty {
            fn from(s: String) -> Self {
                Self::new(s)
            }
        }

        impl std::fmt::Display for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl AsRef<str> for $ty {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

impl_id!(ResourceId);
impl_id!(TagId);
impl_id!(TagCategoryId);

/// Trait for objects that can provide tag information.
/// Used by script execution to check for tags on ships, crew, cargo, etc.
pub trait TagProvider {
    fn has_tag(&self, category: &TagCategoryId, tag: &TagId) -> bool;
}
impl_id!(CardTypeId);
impl_id!(CardId);
impl_id!(CardInstanceId);
impl_id!(SceneId);
impl_id!(LocationId);
impl_id!(FactionId);
impl_id!(FlagId);
impl_id!(ContextId);
impl_id!(SlotId);
impl_id!(ChronicleEntryId);
