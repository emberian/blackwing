use serde::{Deserialize, Serialize};

use crate::{CardId, FactionId, FlagId, GameState, ResourceId, TagCategoryId, TagId, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Requirement {
    MinResource {
        resource: ResourceId,
        value: i64,
    },
    MaxResource {
        resource: ResourceId,
        value: i64,
    },
    HasFlag {
        flag: FlagId,
    },
    NotFlag {
        flag: FlagId,
    },
    FlagEquals {
        flag: FlagId,
        value: Value,
    },
    FlagCompare {
        flag: FlagId,
        op: CompareOp,
        value: i64,
    },
    HasTag {
        category: TagCategoryId,
        tag: TagId,
    },
    HasCard {
        card_id: CardId,
    },
    MinFactionReputation {
        faction: FactionId,
        value: i64,
    },
    MaxFactionReputation {
        faction: FactionId,
        value: i64,
    },
    And(Vec<Requirement>),
    Or(Vec<Requirement>),
    Not(Box<Requirement>),
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

pub trait TagProvider {
    fn has_tag(&self, category: &TagCategoryId, tag: &TagId) -> bool;
}

impl Requirement {
    pub fn check(&self, state: &GameState, tags: &impl TagProvider) -> bool {
        match self {
            Requirement::MinResource { resource, value } => state.resource(resource) >= *value,
            Requirement::MaxResource { resource, value } => state.resource(resource) <= *value,
            Requirement::HasFlag { flag } => state.flag_is_truthy(flag),
            Requirement::NotFlag { flag } => !state.flag_is_truthy(flag),
            Requirement::FlagEquals { flag, value } => state.flag(flag) == value,
            Requirement::FlagCompare { flag, op, value } => {
                let flag_val = state.flag(flag).as_int().unwrap_or(0);
                match op {
                    CompareOp::Eq => flag_val == *value,
                    CompareOp::Ne => flag_val != *value,
                    CompareOp::Lt => flag_val < *value,
                    CompareOp::Le => flag_val <= *value,
                    CompareOp::Gt => flag_val > *value,
                    CompareOp::Ge => flag_val >= *value,
                }
            }
            Requirement::HasTag { category, tag } => tags.has_tag(category, tag),
            Requirement::HasCard { card_id } => state.deck_card_ids().any(|c| c == card_id),
            Requirement::MinFactionReputation { faction, value } => {
                state.faction_reputation(faction) >= *value
            }
            Requirement::MaxFactionReputation { faction, value } => {
                state.faction_reputation(faction) <= *value
            }
            Requirement::And(reqs) => reqs.iter().all(|r| r.check(state, tags)),
            Requirement::Or(reqs) => reqs.iter().any(|r| r.check(state, tags)),
            Requirement::Not(req) => !req.check(state, tags),
            Requirement::Always => true,
            Requirement::Never => false,
        }
    }

    pub fn and(reqs: impl IntoIterator<Item = Requirement>) -> Self {
        let collected: Vec<Requirement> = reqs.into_iter().collect();
        match collected.len() {
            0 => Requirement::Always,
            1 => collected.into_iter().next().unwrap(),
            _ => Requirement::And(collected),
        }
    }

    pub fn or(reqs: impl IntoIterator<Item = Requirement>) -> Self {
        let collected: Vec<Requirement> = reqs.into_iter().collect();
        match collected.len() {
            0 => Requirement::Never,
            1 => collected.into_iter().next().unwrap(),
            _ => Requirement::Or(collected),
        }
    }

    pub fn not(req: Requirement) -> Self {
        match req {
            Requirement::Not(inner) => *inner,
            Requirement::Always => Requirement::Never,
            Requirement::Never => Requirement::Always,
            other => Requirement::Not(Box::new(other)),
        }
    }

    pub fn min_resource(resource: impl Into<ResourceId>, value: i64) -> Self {
        Requirement::MinResource {
            resource: resource.into(),
            value,
        }
    }

    pub fn max_resource(resource: impl Into<ResourceId>, value: i64) -> Self {
        Requirement::MaxResource {
            resource: resource.into(),
            value,
        }
    }

    pub fn has_flag(flag: impl Into<FlagId>) -> Self {
        Requirement::HasFlag { flag: flag.into() }
    }

    pub fn not_flag(flag: impl Into<FlagId>) -> Self {
        Requirement::NotFlag { flag: flag.into() }
    }

    pub fn has_tag(category: impl Into<TagCategoryId>, tag: impl Into<TagId>) -> Self {
        Requirement::HasTag {
            category: category.into(),
            tag: tag.into(),
        }
    }
}

impl Default for Requirement {
    fn default() -> Self {
        Requirement::Always
    }
}
