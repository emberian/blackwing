use std::collections::HashSet;

use engine_core::{GameSchema, Navigation, Scene};

use crate::error::{ValidationError, ValidationReport, ValidationWarning};

pub struct Validator<'a> {
    schema: &'a GameSchema,
}

impl<'a> Validator<'a> {
    pub fn new(schema: &'a GameSchema) -> Self {
        Self { schema }
    }

    pub fn validate(&self, scene: &Scene) -> ValidationReport {
        let mut report = ValidationReport::default();
        let scene_id = scene.id.as_str();

        self.validate_context(scene, &mut report);

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
            passages: vec![Passage {
                text: "Hello".into(),
                choices: vec![Choice {
                    text: "Ok".into(),
                    next: Navigation::End,
                    rhai_condition: "resource(\"credits\") >= 10".into(),
                    rhai_effects: "modify_resource(\"credits\", -10);".into(),
                }],
                rhai_on_enter: None,
            }],
            rhai_requirements: Default::default(),
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
    fn test_unreachable_passage_error() {
        let schema = make_test_schema();
        let mut scene = make_valid_scene();
        // Add an unreachable second passage
        scene.passages.push(Passage {
            text: "Unreachable".into(),
            choices: vec![],
            rhai_on_enter: None,
        });

        let report = validate(&scene, &schema);
        assert!(!report.is_ok());
        assert!(report.errors.iter().any(|e| matches!(
            e,
            ValidationError::UnreachablePassage { passage, .. } if passage == "passage_1"
        )));
    }
}
