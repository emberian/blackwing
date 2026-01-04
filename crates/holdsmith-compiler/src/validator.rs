use std::collections::HashSet;

use engine_core::{ContentRegistry, Effect, GameSchema, Navigation, Requirement, Scene};

use crate::error::{ValidationError, ValidationReport, ValidationWarning};

pub struct Validator<'a> {
    schema: &'a GameSchema,
    cards: Option<&'a dyn ContentRegistry>,
}

impl<'a> Validator<'a> {
    pub fn new(schema: &'a GameSchema) -> Self {
        Self {
            schema,
            cards: None,
        }
    }

    pub fn with_cards(mut self, registry: &'a dyn ContentRegistry) -> Self {
        self.cards = Some(registry);
        self
    }

    pub fn validate(&self, scene: &Scene) -> ValidationReport {
        let mut report = ValidationReport::default();
        let scene_id = scene.id.as_str();

        self.validate_context(scene, &mut report);
        self.validate_requirements(&scene.requirements, scene_id, &mut report);

        let mut reachable_passages = HashSet::new();
        self.find_reachable_passages(scene, 0, &mut reachable_passages);

        for (idx, passage) in scene.passages.iter().enumerate() {
            if idx > 0 && !reachable_passages.contains(&idx) {
                report.error(ValidationError::UnreachablePassage {
                    passage: format!("passage_{}", idx),
                    scene_id: scene_id.to_string(),
                });
            }

            if passage.text.is_empty() && passage.choices.is_empty() {
                report.warn(ValidationWarning::EmptyPassage {
                    passage: format!("passage_{}", idx),
                    scene_id: scene_id.to_string(),
                });
            }

            for choice in &passage.choices {
                self.validate_requirements(&choice.requirements, scene_id, &mut report);

                for effect in &choice.effects {
                    self.validate_effect(effect, scene_id, &mut report);
                }
            }
        }

        report
    }

    fn validate_context(&self, scene: &Scene, report: &mut ValidationReport) {
        if let Some(ref context) = scene.context {
            if self.schema.context(context).is_none() {
                report.error(ValidationError::UnknownContext {
                    context: context.to_string(),
                    scene_id: scene.id.to_string(),
                });
            }
        }
    }

    fn validate_requirements(
        &self,
        req: &Requirement,
        scene_id: &str,
        report: &mut ValidationReport,
    ) {
        match req {
            Requirement::MinResource { resource, .. }
            | Requirement::MaxResource { resource, .. } => {
                if self.schema.resource(resource).is_none() {
                    report.error(ValidationError::UnknownResource {
                        resource: resource.to_string(),
                        scene_id: scene_id.to_string(),
                    });
                }
            }
            Requirement::HasTag { category, tag } => {
                if !self.schema.is_valid_tag(category, tag) {
                    report.error(ValidationError::UnknownTag {
                        tag: tag.to_string(),
                        category: category.to_string(),
                        scene_id: scene_id.to_string(),
                    });
                }
            }
            Requirement::HasCard { card_id } => {
                if let Some(registry) = self.cards {
                    if registry.get_card(card_id).is_none() {
                        report.error(ValidationError::UnknownCard {
                            card_id: card_id.to_string(),
                            scene_id: scene_id.to_string(),
                        });
                    }
                }
            }
            Requirement::MinFactionReputation { .. }
            | Requirement::MaxFactionReputation { .. } => {
                // Factions are not in schema yet, skip validation
            }
            Requirement::And(reqs) | Requirement::Or(reqs) => {
                for r in reqs {
                    self.validate_requirements(r, scene_id, report);
                }
            }
            Requirement::Not(inner) => {
                self.validate_requirements(inner, scene_id, report);
            }
            Requirement::HasFlag { .. }
            | Requirement::NotFlag { .. }
            | Requirement::FlagEquals { .. }
            | Requirement::FlagCompare { .. }
            | Requirement::Always
            | Requirement::Never => {}
        }
    }

    fn validate_effect(&self, effect: &Effect, scene_id: &str, report: &mut ValidationReport) {
        match effect {
            Effect::ModifyResource { resource, .. } | Effect::SetResource { resource, .. } => {
                if self.schema.resource(resource).is_none() {
                    report.error(ValidationError::UnknownResource {
                        resource: resource.to_string(),
                        scene_id: scene_id.to_string(),
                    });
                }
            }
            Effect::Damage { resource, amount } => {
                if self.schema.resource(resource).is_none() {
                    report.error(ValidationError::UnknownResource {
                        resource: resource.to_string(),
                        scene_id: scene_id.to_string(),
                    });
                }
                if *amount > 50 {
                    report.warn(ValidationWarning::HighDamageValue {
                        resource: resource.to_string(),
                        amount: *amount,
                        scene_id: scene_id.to_string(),
                    });
                }
            }
            Effect::AddCard { card_id } => {
                if let Some(registry) = self.cards {
                    if registry.get_card(card_id).is_none() {
                        report.error(ValidationError::UnknownCard {
                            card_id: card_id.to_string(),
                            scene_id: scene_id.to_string(),
                        });
                    }
                }
            }
            Effect::Compound { effects } => {
                for e in effects {
                    self.validate_effect(e, scene_id, report);
                }
            }
            Effect::SetFlag { .. }
            | Effect::RemoveCards { .. }
            | Effect::ModifyFactionReputation { .. }
            | Effect::AddChronicle { .. }
            | Effect::Script { .. }
            | Effect::EquipModule { .. }
            | Effect::UnequipModule { .. }
            | Effect::ModifyStat { .. } => {}
        }
    }

    fn find_reachable_passages(
        &self,
        scene: &Scene,
        current: usize,
        visited: &mut HashSet<usize>,
    ) {
        if visited.contains(&current) {
            return;
        }
        visited.insert(current);

        if let Some(passage) = scene.passages.get(current) {
            for choice in &passage.choices {
                if let Navigation::Passage(next) = choice.next {
                    self.find_reachable_passages(scene, next, visited);
                }
            }
        }
    }
}

pub fn validate(scene: &Scene, schema: &GameSchema) -> ValidationReport {
    Validator::new(schema).validate(scene)
}

pub fn validate_with_cards(
    scene: &Scene,
    schema: &GameSchema,
    cards: &dyn ContentRegistry,
) -> ValidationReport {
    Validator::new(schema).with_cards(cards).validate(scene)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::*;

    fn make_test_schema() -> GameSchema {
        GameSchema {
            name: "test".to_string(),
            version: "1.0".to_string(),
            resources: vec![
                ResourceDef {
                    id: ResourceId::new("credits"),
                    name: "Credits".to_string(),
                    min: Some(0),
                    max: None,
                    default: 100,
                    on_zero: None,
                },
                ResourceDef {
                    id: ResourceId::new("hull"),
                    name: "Hull".to_string(),
                    min: Some(0),
                    max: Some(100),
                    default: 100,
                    on_zero: Some(OnZeroBehavior::GameOver {
                        reason: "destroyed".to_string(),
                    }),
                },
            ],
            tag_categories: vec![TagCategory {
                id: TagCategoryId::new("ship"),
                name: "Ship".to_string(),
                tags: vec![TagId::new("combat"), TagId::new("stealth")],
            }],
            card_types: vec![],
            contexts: vec![ContextDef {
                id: ContextId::new("journey"),
                name: "Journey".to_string(),
            }],
            slot_types: vec![],
            location_statuses: vec![],
            initial_state: InitialStateDef::default(),
        }
    }

    fn make_valid_scene() -> Scene {
        Scene {
            id: SceneId::new("test_scene"),
            title: "Test".into(),
            tags: Tags::new(),
            context: Some(ContextId::new("journey")),
            weight: 10,
            cooldown: 5,
            requirements: Requirement::Always,
            passages: vec![Passage {
                text: "Hello".into(),
                choices: vec![Choice {
                    text: "Ok".into(),
                    requirements: Requirement::MinResource {
                        resource: ResourceId::new("credits"),
                        value: 10,
                    },
                    effects: vec![Effect::ModifyResource {
                        resource: ResourceId::new("credits"),
                        delta: -10,
                    }],
                    next: Navigation::End,
                }],
            }],
        }
    }

    #[test]
    fn test_valid_scene_passes() {
        let schema = make_test_schema();
        let scene = make_valid_scene();
        let report = validate(&scene, &schema);
        assert!(report.is_ok());
    }

    #[test]
    fn test_unknown_resource_error() {
        let schema = make_test_schema();
        let mut scene = make_valid_scene();
        scene.passages[0].choices[0].effects = vec![Effect::ModifyResource {
            resource: ResourceId::new("nonexistent"),
            delta: 10,
        }];

        let report = validate(&scene, &schema);
        assert!(!report.is_ok());
        assert!(report.errors.iter().any(|e| matches!(
            e,
            ValidationError::UnknownResource { resource, .. } if resource == "nonexistent"
        )));
    }

    #[test]
    fn test_unknown_context_error() {
        let schema = make_test_schema();
        let mut scene = make_valid_scene();
        scene.context = Some(ContextId::new("invalid_context"));

        let report = validate(&scene, &schema);
        assert!(!report.is_ok());
        assert!(report.errors.iter().any(|e| matches!(
            e,
            ValidationError::UnknownContext { context, .. } if context == "invalid_context"
        )));
    }

    #[test]
    fn test_unknown_tag_error() {
        let schema = make_test_schema();
        let mut scene = make_valid_scene();
        scene.passages[0].choices[0].requirements = Requirement::HasTag {
            category: TagCategoryId::new("ship"),
            tag: TagId::new("nonexistent_tag"),
        };

        let report = validate(&scene, &schema);
        assert!(!report.is_ok());
    }

    #[test]
    fn test_high_damage_warning() {
        let schema = make_test_schema();
        let mut scene = make_valid_scene();
        scene.passages[0].choices[0].effects = vec![Effect::Damage {
            resource: ResourceId::new("hull"),
            amount: 100,
        }];

        let report = validate(&scene, &schema);
        assert!(report.is_ok()); // warnings don't make it fail
        assert!(report.has_warnings());
    }
}
