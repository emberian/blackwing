//! Dialogue trees for NPC conversations.
//!
//! Dialogue trees are separate from scenes for reusability - the same dialogue
//! can be triggered from multiple scenes or by interacting with an NPC directly.

use engine_primitives::EntityId;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A reusable dialogue tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTree {
    /// Unique identifier for this dialogue
    pub id: SmolStr,

    /// Default speaker entity (can be overridden per node)
    pub default_speaker: Option<EntityId>,

    /// The nodes in this dialogue
    pub nodes: Vec<DialogueNode>,
}

impl DialogueTree {
    /// Create a new dialogue tree
    pub fn new(id: impl Into<SmolStr>) -> Self {
        Self {
            id: id.into(),
            default_speaker: None,
            nodes: Vec::new(),
        }
    }

    /// Set the default speaker
    pub fn with_speaker(mut self, speaker: EntityId) -> Self {
        self.default_speaker = Some(speaker);
        self
    }

    /// Add a node to the dialogue
    pub fn with_node(mut self, node: DialogueNode) -> Self {
        self.nodes.push(node);
        self
    }

    /// Get a node by its ID
    pub fn get_node(&self, node_id: &str) -> Option<&DialogueNode> {
        self.nodes.iter().find(|n| n.id == node_id)
    }

    /// Get the start node (first node or node with id "start")
    pub fn start_node(&self) -> Option<&DialogueNode> {
        self.get_node("start").or_else(|| self.nodes.first())
    }
}

/// A single node in a dialogue tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    /// Node identifier (for jumping to this node)
    pub id: SmolStr,

    /// The speaker for this node (overrides dialogue default)
    pub speaker: Option<EntityId>,

    /// The text content (supports ${expression} interpolation)
    pub text: SmolStr,

    /// Available responses/choices
    pub responses: Vec<DialogueResponse>,

    /// Rhai script to run when entering this node
    pub on_enter: Option<SmolStr>,

    /// Rhai script to run when leaving this node
    pub on_exit: Option<SmolStr>,

    /// Condition for this node to be available (Rhai expression)
    pub condition: Option<SmolStr>,
}

impl DialogueNode {
    /// Create a new dialogue node
    pub fn new(id: impl Into<SmolStr>, text: impl Into<SmolStr>) -> Self {
        Self {
            id: id.into(),
            speaker: None,
            text: text.into(),
            responses: Vec::new(),
            on_enter: None,
            on_exit: None,
            condition: None,
        }
    }

    /// Set the speaker
    pub fn with_speaker(mut self, speaker: EntityId) -> Self {
        self.speaker = Some(speaker);
        self
    }

    /// Add a response option
    pub fn with_response(mut self, response: DialogueResponse) -> Self {
        self.responses.push(response);
        self
    }

    /// Set the on_enter script
    pub fn with_on_enter(mut self, script: impl Into<SmolStr>) -> Self {
        self.on_enter = Some(script.into());
        self
    }

    /// Set a condition for this node
    pub fn with_condition(mut self, condition: impl Into<SmolStr>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    /// Check if this is a terminal node (no responses)
    pub fn is_terminal(&self) -> bool {
        self.responses.is_empty()
    }
}

/// A response option in a dialogue node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueResponse {
    /// Display text for this response
    pub text: SmolStr,

    /// Node to jump to when selected (None = end dialogue)
    pub next_node: Option<SmolStr>,

    /// Condition for this response to be available (Rhai expression)
    pub condition: Option<SmolStr>,

    /// Rhai script to run when this response is selected
    pub on_select: Option<SmolStr>,
}

impl DialogueResponse {
    /// Create a new response
    pub fn new(text: impl Into<SmolStr>) -> Self {
        Self {
            text: text.into(),
            next_node: None,
            condition: None,
            on_select: None,
        }
    }

    /// Set the next node to jump to
    pub fn to_node(mut self, node_id: impl Into<SmolStr>) -> Self {
        self.next_node = Some(node_id.into());
        self
    }

    /// End the dialogue when selected
    pub fn end_dialogue(mut self) -> Self {
        self.next_node = None;
        self
    }

    /// Set a condition for this response
    pub fn with_condition(mut self, condition: impl Into<SmolStr>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    /// Set the on_select script
    pub fn with_on_select(mut self, script: impl Into<SmolStr>) -> Self {
        self.on_select = Some(script.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogue_tree_builder() {
        let dialogue = DialogueTree::new("bob_greeting")
            .with_speaker(EntityId::new("npc", "merchant_bob"))
            .with_node(
                DialogueNode::new("start", "Welcome to my shop, traveler!")
                    .with_response(DialogueResponse::new("Browse wares").to_node("shop"))
                    .with_response(
                        DialogueResponse::new("Ask about rumors")
                            .to_node("rumors")
                            .with_condition("reputation('merchants') >= 10"),
                    )
                    .with_response(DialogueResponse::new("Leave").end_dialogue()),
            )
            .with_node(
                DialogueNode::new("shop", "Take a look at what I have.")
                    .with_on_enter("open_shop('bob_inventory');"),
            )
            .with_node(
                DialogueNode::new("rumors", "I heard there's trouble at the old mine...")
                    .with_on_enter("set_flag('heard_mine_rumor', true);"),
            );

        assert_eq!(dialogue.id, "bob_greeting");
        assert!(dialogue.start_node().is_some());
        assert_eq!(dialogue.nodes.len(), 3);

        let start = dialogue.start_node().unwrap();
        assert_eq!(start.responses.len(), 3);
    }
}
