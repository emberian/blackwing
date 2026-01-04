use serde::{Deserialize, Serialize};

use crate::{CardId, CardInstanceId, Effect, LocationId, SceneId, SlotId};

/// All commands that can be dispatched to the game engine.
/// Commands represent player actions and system operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    // === Travel & Navigation ===
    /// Initiate travel to a destination
    Travel { destination: LocationId },

    // === Scene/Event Commands ===
    /// Make a choice in an active scene
    MakeChoice {
        scene_id: SceneId,
        passage_index: usize,
        choice_index: usize,
    },

    /// Continue/exit a scene without making a choice
    ContinueScene { scene_id: SceneId },

    /// Force-trigger a specific scene (for testing/scripting)
    TriggerScene { scene_id: SceneId },

    // === Trade Commands ===
    /// Buy cards from the current port's market
    TradeBuy { card_id: CardId, quantity: u32 },

    /// Sell a card instance
    TradeSell { instance_id: CardInstanceId },

    // === Card Management ===
    /// Equip a card to the active deck
    CardEquip { instance_id: CardInstanceId },

    /// Unequip a card from the active deck
    CardUnequip { instance_id: CardInstanceId },

    /// Upgrade a card (if it has an upgrade path)
    CardUpgrade { instance_id: CardInstanceId },

    // === Module Management ===
    /// Install a module into a ship slot
    ModuleInstall {
        instance_id: CardInstanceId,
        slot_id: SlotId,
    },

    /// Uninstall a module from a ship slot
    ModuleUninstall { slot_id: SlotId },

    // === Contract Commands ===
    /// Accept a contract from the current port
    ContractAccept { card_id: CardId },

    /// Complete an active contract (at destination with cargo)
    ContractComplete { instance_id: CardInstanceId },

    /// Abandon an active contract (with penalty)
    ContractAbandon { instance_id: CardInstanceId },

    // === Crew Commands ===
    /// Hire a crew member at the current port
    CrewHire { card_id: CardId },

    /// Dismiss a crew member
    CrewDismiss { instance_id: CardInstanceId },

    // === Port Services ===
    /// Repair hull damage (costs credits)
    Repair { amount: i64 },

    /// Purchase supplies (costs credits)
    Resupply { amount: i64 },

    /// Purchase fuel (costs credits)
    Refuel { amount: i64 },

    // === System Commands ===
    /// Apply a list of effects directly
    ApplyEffects { effects: Vec<Effect>, reason: String },

    /// Advance the game cycle
    AdvanceCycle,

    /// Initialize a new game
    InitializeGame,
}

impl Command {
    // === Factory methods for cleaner API ===

    pub fn travel(destination: impl Into<LocationId>) -> Self {
        Command::Travel {
            destination: destination.into(),
        }
    }

    pub fn make_choice(scene_id: SceneId, passage_index: usize, choice_index: usize) -> Self {
        Command::MakeChoice {
            scene_id,
            passage_index,
            choice_index,
        }
    }

    pub fn trigger_scene(scene_id: SceneId) -> Self {
        Command::TriggerScene { scene_id }
    }

    pub fn trade_buy(card_id: impl Into<CardId>, quantity: u32) -> Self {
        Command::TradeBuy {
            card_id: card_id.into(),
            quantity,
        }
    }

    pub fn trade_sell(instance_id: CardInstanceId) -> Self {
        Command::TradeSell { instance_id }
    }

    pub fn card_equip(instance_id: CardInstanceId) -> Self {
        Command::CardEquip { instance_id }
    }

    pub fn card_unequip(instance_id: CardInstanceId) -> Self {
        Command::CardUnequip { instance_id }
    }

    pub fn module_install(instance_id: CardInstanceId, slot_id: impl Into<SlotId>) -> Self {
        Command::ModuleInstall {
            instance_id,
            slot_id: slot_id.into(),
        }
    }

    pub fn module_uninstall(slot_id: impl Into<SlotId>) -> Self {
        Command::ModuleUninstall {
            slot_id: slot_id.into(),
        }
    }

    pub fn contract_accept(card_id: impl Into<CardId>) -> Self {
        Command::ContractAccept {
            card_id: card_id.into(),
        }
    }

    pub fn repair(amount: i64) -> Self {
        Command::Repair { amount }
    }

    pub fn resupply(amount: i64) -> Self {
        Command::Resupply { amount }
    }

    pub fn refuel(amount: i64) -> Self {
        Command::Refuel { amount }
    }

    pub fn apply_effects(effects: Vec<Effect>, reason: impl Into<String>) -> Self {
        Command::ApplyEffects {
            effects,
            reason: reason.into(),
        }
    }
}
