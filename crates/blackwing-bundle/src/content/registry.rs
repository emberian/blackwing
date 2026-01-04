//! Content registry for accessing game content.
//!
//! The registry provides a unified interface for accessing all game content:
//! - Entity templates
//! - Dialogue trees
//! - Quest definitions

use blackwing_core::EntityId;
use smol_str::SmolStr;

use super::dialogue::DialogueTree;
use super::quest::QuestDef;
use super::template::EntityTemplate;

/// Registry for accessing game content.
///
/// Implementations should be thread-safe (Send + Sync) for use in
/// multithreaded contexts.
pub trait ContentRegistry: Send + Sync {
    /// Get an entity template by ID
    fn get_template(&self, id: &EntityId) -> Option<&EntityTemplate>;

    /// Get a dialogue tree by ID
    fn get_dialogue(&self, id: &str) -> Option<&DialogueTree>;

    /// Get a quest definition by ID
    fn get_quest(&self, id: &str) -> Option<&QuestDef>;

    /// Iterate over all entity templates
    fn templates(&self) -> Box<dyn Iterator<Item = &EntityTemplate> + '_>;

    /// Iterate over all dialogue trees
    fn dialogues(&self) -> Box<dyn Iterator<Item = &DialogueTree> + '_>;

    /// Iterate over all quest definitions
    fn quests(&self) -> Box<dyn Iterator<Item = &QuestDef> + '_>;

    /// Get templates of a specific kind
    fn templates_of_kind(&self, kind: &str) -> Box<dyn Iterator<Item = &EntityTemplate> + '_>;
}

/// A simple in-memory content registry.
///
/// Useful for testing and simple games that load all content at startup.
#[derive(Debug, Default)]
pub struct SimpleRegistry {
    templates: indexmap::IndexMap<String, EntityTemplate>,
    dialogues: indexmap::IndexMap<SmolStr, DialogueTree>,
    quests: indexmap::IndexMap<SmolStr, QuestDef>,
}

impl SimpleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an entity template
    pub fn add_template(&mut self, template: EntityTemplate) {
        self.templates
            .insert(template.id.as_qualified(), template);
    }

    /// Add a dialogue tree
    pub fn add_dialogue(&mut self, dialogue: DialogueTree) {
        self.dialogues.insert(dialogue.id.clone(), dialogue);
    }

    /// Add a quest definition
    pub fn add_quest(&mut self, quest: QuestDef) {
        self.quests.insert(quest.id.clone(), quest);
    }
}

impl ContentRegistry for SimpleRegistry {
    fn get_template(&self, id: &EntityId) -> Option<&EntityTemplate> {
        self.templates.get(&id.as_qualified())
    }

    fn get_dialogue(&self, id: &str) -> Option<&DialogueTree> {
        self.dialogues.get(id)
    }

    fn get_quest(&self, id: &str) -> Option<&QuestDef> {
        self.quests.get(id)
    }

    fn templates(&self) -> Box<dyn Iterator<Item = &EntityTemplate> + '_> {
        Box::new(self.templates.values())
    }

    fn dialogues(&self) -> Box<dyn Iterator<Item = &DialogueTree> + '_> {
        Box::new(self.dialogues.values())
    }

    fn quests(&self) -> Box<dyn Iterator<Item = &QuestDef> + '_> {
        Box::new(self.quests.values())
    }

    fn templates_of_kind(&self, kind: &str) -> Box<dyn Iterator<Item = &EntityTemplate> + '_> {
        let kind = kind.to_string();
        Box::new(self.templates.values().filter(move |t| t.kind() == kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blackwing_core::Value;

    #[test]
    fn simple_registry() {
        let mut registry = SimpleRegistry::new();

        // Add a template
        registry.add_template(
            EntityTemplate::new(EntityId::new("npc", "bob"))
                .with_component("name", Value::String("Bob".into())),
        );

        // Add a dialogue
        registry.add_dialogue(DialogueTree::new("bob_greeting"));

        // Add a quest
        registry.add_quest(QuestDef::new("test_quest", "Test Quest"));

        // Query
        assert!(registry.get_template(&EntityId::new("npc", "bob")).is_some());
        assert!(registry.get_dialogue("bob_greeting").is_some());
        assert!(registry.get_quest("test_quest").is_some());

        // Iteration
        assert_eq!(registry.templates().count(), 1);
        assert_eq!(registry.dialogues().count(), 1);
        assert_eq!(registry.quests().count(), 1);
        assert_eq!(registry.templates_of_kind("npc").count(), 1);
        assert_eq!(registry.templates_of_kind("item").count(), 0);
    }
}
