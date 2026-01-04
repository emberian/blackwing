//! Quest definitions for tracking objectives and rewards.

use engine_primitives::{EntityId, Value};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A quest definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestDef {
    /// Unique quest identifier
    pub id: SmolStr,

    /// Display title
    pub title: SmolStr,

    /// Description text
    pub description: SmolStr,

    /// Quest stages in order
    pub stages: Vec<QuestStage>,

    /// Rhai script to run when quest starts
    pub on_start: Option<SmolStr>,

    /// Rhai script to run when quest completes
    pub on_complete: Option<SmolStr>,

    /// Rhai script to run when quest fails
    pub on_fail: Option<SmolStr>,

    /// Whether this quest can be repeated
    pub repeatable: bool,

    /// Prerequisites (other quest IDs that must be completed)
    pub prerequisites: Vec<SmolStr>,
}

impl QuestDef {
    /// Create a new quest definition
    pub fn new(id: impl Into<SmolStr>, title: impl Into<SmolStr>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: SmolStr::default(),
            stages: Vec::new(),
            on_start: None,
            on_complete: None,
            on_fail: None,
            repeatable: false,
            prerequisites: Vec::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<SmolStr>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a stage
    pub fn with_stage(mut self, stage: QuestStage) -> Self {
        self.stages.push(stage);
        self
    }

    /// Set the on_start script
    pub fn with_on_start(mut self, script: impl Into<SmolStr>) -> Self {
        self.on_start = Some(script.into());
        self
    }

    /// Set the on_complete script
    pub fn with_on_complete(mut self, script: impl Into<SmolStr>) -> Self {
        self.on_complete = Some(script.into());
        self
    }

    /// Set repeatable
    pub fn repeatable(mut self) -> Self {
        self.repeatable = true;
        self
    }

    /// Add a prerequisite
    pub fn requires(mut self, quest_id: impl Into<SmolStr>) -> Self {
        self.prerequisites.push(quest_id.into());
        self
    }

    /// Get a stage by ID
    pub fn get_stage(&self, stage_id: &str) -> Option<&QuestStage> {
        self.stages.iter().find(|s| s.id == stage_id)
    }

    /// Get the first stage
    pub fn first_stage(&self) -> Option<&QuestStage> {
        self.stages.first()
    }
}

/// A stage in a quest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestStage {
    /// Stage identifier
    pub id: SmolStr,

    /// Display text for this stage
    pub text: SmolStr,

    /// Objectives to complete this stage
    pub objectives: Vec<Objective>,

    /// Rhai script to run when stage is entered
    pub on_enter: Option<SmolStr>,

    /// Rhai script to run when stage is completed
    pub on_complete: Option<SmolStr>,

    /// Previous stages that must be completed
    pub requires: Vec<SmolStr>,

    /// Custom completion check (Rhai expression)
    pub completion_check: Option<SmolStr>,
}

impl QuestStage {
    /// Create a new quest stage
    pub fn new(id: impl Into<SmolStr>, text: impl Into<SmolStr>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            objectives: Vec::new(),
            on_enter: None,
            on_complete: None,
            requires: Vec::new(),
            completion_check: None,
        }
    }

    /// Add an objective
    pub fn with_objective(mut self, objective: Objective) -> Self {
        self.objectives.push(objective);
        self
    }

    /// Set the on_enter script
    pub fn with_on_enter(mut self, script: impl Into<SmolStr>) -> Self {
        self.on_enter = Some(script.into());
        self
    }

    /// Set the on_complete script
    pub fn with_on_complete(mut self, script: impl Into<SmolStr>) -> Self {
        self.on_complete = Some(script.into());
        self
    }

    /// Add a stage requirement
    pub fn requires_stage(mut self, stage_id: impl Into<SmolStr>) -> Self {
        self.requires.push(stage_id.into());
        self
    }

    /// Set a custom completion check
    pub fn with_completion_check(mut self, check: impl Into<SmolStr>) -> Self {
        self.completion_check = Some(check.into());
        self
    }
}

/// An objective within a quest stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Objective {
    /// Visit a specific location
    Visit {
        location: EntityId,
        text: SmolStr,
    },

    /// Interact with an entity
    Interact {
        target: EntityId,
        text: SmolStr,
    },

    /// Collect items
    Collect {
        item: EntityId,
        count: u32,
        text: SmolStr,
    },

    /// Defeat entities
    Defeat {
        target_kind: SmolStr,
        target_tag: Option<SmolStr>,
        count: u32,
        text: SmolStr,
    },

    /// Set a flag to a value
    Flag {
        flag: SmolStr,
        value: Value,
        text: SmolStr,
    },

    /// Custom objective with Rhai check
    Custom {
        id: SmolStr,
        text: SmolStr,
        check: SmolStr,
    },
}

impl Objective {
    /// Create a visit objective
    pub fn visit(location: EntityId, text: impl Into<SmolStr>) -> Self {
        Self::Visit {
            location,
            text: text.into(),
        }
    }

    /// Create an interact objective
    pub fn interact(target: EntityId, text: impl Into<SmolStr>) -> Self {
        Self::Interact {
            target,
            text: text.into(),
        }
    }

    /// Create a collect objective
    pub fn collect(item: EntityId, count: u32, text: impl Into<SmolStr>) -> Self {
        Self::Collect {
            item,
            count,
            text: text.into(),
        }
    }

    /// Create a defeat objective
    pub fn defeat(
        target_kind: impl Into<SmolStr>,
        count: u32,
        text: impl Into<SmolStr>,
    ) -> Self {
        Self::Defeat {
            target_kind: target_kind.into(),
            target_tag: None,
            count,
            text: text.into(),
        }
    }

    /// Create a custom objective
    pub fn custom(
        id: impl Into<SmolStr>,
        text: impl Into<SmolStr>,
        check: impl Into<SmolStr>,
    ) -> Self {
        Self::Custom {
            id: id.into(),
            text: text.into(),
            check: check.into(),
        }
    }

    /// Get the display text for this objective
    pub fn text(&self) -> &str {
        match self {
            Self::Visit { text, .. } => text,
            Self::Interact { text, .. } => text,
            Self::Collect { text, .. } => text,
            Self::Defeat { text, .. } => text,
            Self::Flag { text, .. } => text,
            Self::Custom { text, .. } => text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_builder() {
        let quest = QuestDef::new("rescue_miner", "Rescue the Trapped Miner")
            .with_description("A miner is trapped in the old mine. Go rescue them!")
            .with_on_start("add_marker('location:old_mine');")
            .with_stage(
                QuestStage::new("find_mine", "Travel to the old mine")
                    .with_objective(Objective::visit(
                        EntityId::new("location", "old_mine"),
                        "Visit the old mine",
                    )),
            )
            .with_stage(
                QuestStage::new("rescue", "Find and rescue the miner")
                    .requires_stage("find_mine")
                    .with_objective(Objective::interact(
                        EntityId::new("npc", "trapped_miner"),
                        "Talk to the trapped miner",
                    )),
            )
            .with_stage(
                QuestStage::new("return", "Return to Bob")
                    .requires_stage("rescue")
                    .with_objective(Objective::visit(
                        EntityId::new("location", "market_square"),
                        "Return to Bob in the market square",
                    )),
            )
            .with_on_complete("modify_resource('gold', 100);");

        assert_eq!(quest.id, "rescue_miner");
        assert_eq!(quest.stages.len(), 3);
        assert!(quest.on_start.is_some());
        assert!(quest.on_complete.is_some());

        let first = quest.first_stage().unwrap();
        assert_eq!(first.id, "find_mine");
    }
}
