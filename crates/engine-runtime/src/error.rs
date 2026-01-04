use thiserror::Error;

use engine_core::{CardId, CardInstanceId, LocationId, SceneId, SlotId};

#[derive(Debug, Error)]
pub enum RuntimeError {
    // === Scene/Choice Errors ===
    #[error("scene not found: {scene_id}")]
    SceneNotFound { scene_id: SceneId },

    #[error("passage not found: {scene_id} passage {passage_index}")]
    PassageNotFound {
        scene_id: SceneId,
        passage_index: usize,
    },

    #[error("choice not found: {scene_id} passage {passage_index} choice {choice_index}")]
    ChoiceNotFound {
        scene_id: SceneId,
        passage_index: usize,
        choice_index: usize,
    },

    #[error("choice requirements not met")]
    ChoiceRequirementsNotMet,

    #[error("no active scene")]
    NoActiveScene,

    // === Card Errors ===
    #[error("card definition not found: {card_id}")]
    CardNotFound { card_id: CardId },

    #[error("card instance not found: {instance_id}")]
    CardInstanceNotFound { instance_id: CardInstanceId },

    #[error("card not available at current port: {card_id}")]
    CardNotAvailable { card_id: CardId },

    #[error("card already equipped: {instance_id}")]
    AlreadyEquipped { instance_id: CardInstanceId },

    #[error("card not equipped: {instance_id}")]
    NotEquipped { instance_id: CardInstanceId },

    #[error("card cannot be upgraded")]
    CannotUpgrade,

    // === Module Errors ===
    #[error("card is not a module: {card_id}")]
    NotAModule { card_id: CardId },

    #[error("slot is occupied: {slot_id}")]
    SlotOccupied { slot_id: SlotId },

    #[error("slot is empty: {slot_id}")]
    SlotEmpty { slot_id: SlotId },

    // === Contract Errors ===
    #[error("card is not a contract: {card_id}")]
    NotAContract { card_id: CardId },

    #[error("invalid contract")]
    InvalidContract,

    #[error("not at contract destination: {required}")]
    NotAtDestination { required: LocationId },

    #[error("insufficient cargo: need {required}, have {available}")]
    InsufficientCargo { required: u32, available: u32 },

    // === Crew Errors ===
    #[error("card is not a crew member: {card_id}")]
    NotACrew { card_id: CardId },

    // === Resource Errors ===
    #[error("insufficient credits: need {required}, have {available}")]
    InsufficientCredits { required: i64, available: i64 },

    #[error("insufficient fuel: need {required}, have {available}")]
    InsufficientFuel { required: i64, available: i64 },

    // === Location Errors ===
    #[error("not at a port")]
    NotAtPort,

    // === Game State Errors ===
    #[error("game over: {reason}")]
    GameOver { reason: String },

    // === Script Errors ===
    #[error("script error: {message}")]
    ScriptError { message: String },

    // === Journey Errors ===
    #[error("no journey in progress")]
    NoJourneyInProgress,
}
