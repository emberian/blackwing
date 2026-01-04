//! UI node types for declarative UI composition.

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A declarative UI node that systems can emit.
///
/// These nodes describe the UI structure without specifying rendering details.
/// The frontend (e.g., blackwing-ui) is responsible for rendering these to
/// actual UI components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiNode {
    // === Layout nodes ===
    /// Vertical stack of children
    VStack {
        children: Vec<UiNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        gap: Option<f32>,
    },

    /// Horizontal stack of children
    HStack {
        children: Vec<UiNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        gap: Option<f32>,
    },

    /// Card container with optional title
    Card {
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<SmolStr>,
        children: Vec<UiNode>,
    },

    // === Content nodes ===
    /// Passage of narrative text (supports markdown)
    Passage { text: SmolStr },

    /// List of choices for the player
    Choices { items: Vec<UiChoice> },

    /// Resource bar showing one or more resources
    ResourceBar { resources: Vec<UiResource> },

    /// Simple text display
    Text { text: SmolStr },

    /// Image display
    Image {
        src: SmolStr,
        #[serde(skip_serializing_if = "Option::is_none")]
        alt: Option<SmolStr>,
    },

    // === Interactive nodes ===
    /// Clickable button
    Button {
        text: SmolStr,
        /// Command to dispatch when clicked (JSON string)
        action: SmolStr,
        #[serde(default = "default_true")]
        enabled: bool,
    },

    /// Text input field
    Input {
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<SmolStr>,
        /// World state path to bind the value to
        binding: SmolStr,
    },

    // === Reactive/binding nodes ===
    /// Bind to a world state path, re-render when it changes
    Bound {
        /// Path like "resource:gold" or "flag:visited_town"
        path: SmolStr,
        /// Fallback node if path doesn't exist
        fallback: Box<UiNode>,
    },

    /// Conditional rendering based on a Rhai expression
    Conditional {
        /// Rhai expression that evaluates to bool
        condition: SmolStr,
        then_node: Box<UiNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        else_node: Option<Box<UiNode>>,
    },

    /// Iterate over a collection
    ForEach {
        /// Variable name for each item
        item_var: SmolStr,
        /// World state path to array
        items_path: SmolStr,
        /// Template node (rendered for each item)
        template: Box<UiNode>,
    },

    // === Special nodes ===
    /// Empty node (renders nothing)
    Empty,

    /// Spacer that takes up available space
    Spacer,

    /// Divider line
    Divider,
}

fn default_true() -> bool {
    true
}

impl UiNode {
    /// Create a vertical stack
    pub fn vstack(children: Vec<UiNode>) -> Self {
        Self::VStack { children, gap: None }
    }

    /// Create a horizontal stack
    pub fn hstack(children: Vec<UiNode>) -> Self {
        Self::HStack { children, gap: None }
    }

    /// Create a passage
    pub fn passage(text: impl Into<SmolStr>) -> Self {
        Self::Passage { text: text.into() }
    }

    /// Create a text node
    pub fn text(text: impl Into<SmolStr>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Create choices from a list
    pub fn choices(items: Vec<UiChoice>) -> Self {
        Self::Choices { items }
    }

    /// Create a button
    pub fn button(text: impl Into<SmolStr>, action: impl Into<SmolStr>) -> Self {
        Self::Button {
            text: text.into(),
            action: action.into(),
            enabled: true,
        }
    }

    /// Create a card
    pub fn card(title: Option<SmolStr>, children: Vec<UiNode>) -> Self {
        Self::Card { title, children }
    }

    /// Create an empty node
    pub fn empty() -> Self {
        Self::Empty
    }
}

/// A choice option for the player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiChoice {
    /// Display text for the choice
    pub text: SmolStr,
    /// Command to dispatch when selected (JSON string or command kind)
    pub action: SmolStr,
    /// Whether the choice is currently available
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Reason the choice is disabled (shown as tooltip)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<SmolStr>,
}

impl UiChoice {
    /// Create an enabled choice
    pub fn new(text: impl Into<SmolStr>, action: impl Into<SmolStr>) -> Self {
        Self {
            text: text.into(),
            action: action.into(),
            enabled: true,
            disabled_reason: None,
        }
    }

    /// Create a disabled choice with a reason
    pub fn disabled(
        text: impl Into<SmolStr>,
        action: impl Into<SmolStr>,
        reason: impl Into<SmolStr>,
    ) -> Self {
        Self {
            text: text.into(),
            action: action.into(),
            enabled: false,
            disabled_reason: Some(reason.into()),
        }
    }
}

/// Resource display configuration for ResourceBar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiResource {
    /// Resource identifier
    pub id: SmolStr,
    /// Display label
    pub label: SmolStr,
    /// Optional icon name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<SmolStr>,
    /// Current value
    pub current: i64,
    /// Maximum value (for showing as bar)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,
    /// Warning threshold (resource turns yellow below this)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning_threshold: Option<i64>,
    /// Critical threshold (resource turns red below this)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical_threshold: Option<i64>,
}

impl UiResource {
    /// Create a simple resource display
    pub fn new(id: impl Into<SmolStr>, label: impl Into<SmolStr>, current: i64) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            current,
            max: None,
            warning_threshold: None,
            critical_threshold: None,
        }
    }

    /// Create a resource with max (displayed as a bar)
    pub fn with_max(
        id: impl Into<SmolStr>,
        label: impl Into<SmolStr>,
        current: i64,
        max: i64,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            current,
            max: Some(max),
            warning_threshold: None,
            critical_threshold: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_node_serialization() {
        let node = UiNode::vstack(vec![
            UiNode::passage("Hello, world!"),
            UiNode::choices(vec![
                UiChoice::new("Option A", "choose_a"),
                UiChoice::disabled("Option B", "choose_b", "Not enough gold"),
            ]),
        ]);

        let json = serde_json::to_string_pretty(&node).unwrap();
        assert!(json.contains("v_stack"));
        assert!(json.contains("Hello, world!"));

        // Round-trip
        let parsed: UiNode = serde_json::from_str(&json).unwrap();
        assert_eq!(node, parsed);
    }

    #[test]
    fn ui_choice_creation() {
        let enabled = UiChoice::new("Click me", "do_thing");
        assert!(enabled.enabled);
        assert!(enabled.disabled_reason.is_none());

        let disabled = UiChoice::disabled("Can't click", "do_thing", "Missing key");
        assert!(!disabled.enabled);
        assert_eq!(disabled.disabled_reason.as_deref(), Some("Missing key"));
    }
}
