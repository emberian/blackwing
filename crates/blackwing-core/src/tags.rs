//! Tag system for entities.
//!
//! Tags are simple string labels that can be attached to entities for filtering
//! and conditional logic. They're stored efficiently using SmallVec.

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::TagId;

/// A collection of tags, optimized for small numbers (up to 4 inline).
pub type Tags = SmallVec<[TagId; 4]>;

/// Extension trait for Tags operations
pub trait TagsExt {
    fn has_tag(&self, tag: &str) -> bool;
    fn add_tag(&mut self, tag: impl Into<TagId>);
    fn remove_tag(&mut self, tag: &str) -> bool;
    fn from_slice(tags: &[&str]) -> Self;
}

impl TagsExt for Tags {
    fn has_tag(&self, tag: &str) -> bool {
        self.iter().any(|t| t.as_str() == tag)
    }

    fn add_tag(&mut self, tag: impl Into<TagId>) {
        let tag = tag.into();
        if !self.iter().any(|t| t.as_str() == tag.as_str()) {
            self.push(tag);
        }
    }

    fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.iter().position(|t| t.as_str() == tag) {
            SmallVec::remove(self, pos);
            true
        } else {
            false
        }
    }

    fn from_slice(tags: &[&str]) -> Self {
        tags.iter().map(|&s| TagId::new(s)).collect()
    }
}

/// Categorized tags (e.g., "ship:combat", "cargo:volatile")
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategorizedTags {
    pub categories: std::collections::HashMap<SmolStr, Tags>,
}

impl CategorizedTags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has(&self, category: &str, tag: &str) -> bool {
        self.categories
            .get(category)
            .map(|tags| tags.has_tag(tag))
            .unwrap_or(false)
    }

    pub fn add(&mut self, category: impl Into<SmolStr>, tag: impl Into<TagId>) {
        self.categories
            .entry(category.into())
            .or_default()
            .add_tag(tag);
    }

    pub fn remove(&mut self, category: &str, tag: &str) -> bool {
        self.categories
            .get_mut(category)
            .map(|tags| tags.remove_tag(tag))
            .unwrap_or(false)
    }

    pub fn tags_in(&self, category: &str) -> Option<&Tags> {
        self.categories.get(category)
    }

    pub fn all_tags(&self) -> impl Iterator<Item = (&str, &str)> {
        self.categories.iter().flat_map(|(cat, tags)| {
            tags.iter()
                .map(move |tag| (cat.as_str(), tag.as_str()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_basic_operations() {
        let mut tags = Tags::new();

        tags.add_tag("hostile");
        assert!(tags.has_tag("hostile"));
        assert!(!tags.has_tag("friendly"));

        tags.add_tag("armed");
        assert!(tags.has_tag("armed"));

        // Adding duplicate does nothing
        tags.add_tag("hostile");
        assert_eq!(tags.len(), 2);

        assert!(tags.remove_tag("hostile"));
        assert!(!tags.has_tag("hostile"));
        assert!(!tags.remove_tag("nonexistent"));
    }

    #[test]
    fn categorized_tags() {
        let mut tags = CategorizedTags::new();

        tags.add("ship", "combat");
        tags.add("ship", "stealth");
        tags.add("cargo", "volatile");

        assert!(tags.has("ship", "combat"));
        assert!(tags.has("ship", "stealth"));
        assert!(tags.has("cargo", "volatile"));
        assert!(!tags.has("ship", "volatile"));
        assert!(!tags.has("crew", "combat"));
    }
}
