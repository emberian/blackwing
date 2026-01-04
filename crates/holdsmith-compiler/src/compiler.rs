use std::collections::HashMap;

use engine_core::{
    Choice as RuntimeChoice, ContextId, Effect, FlagId, Navigation, Passage as RuntimePassage,
    Requirement, ResourceId, Scene, SceneId, TagCategoryId, TagId, Tags, Value,
};
use holdsmith_parser::{
    self as ast, AssignOp, CompareOp, Condition, ConditionClause, FlagCondition, FlagValue,
    Passage, PassageContent, SceneFile, TagSource,
};
use smol_str::SmolStr;

use crate::codegen;
use crate::error::{CompileError, CompileResult};

pub struct Compiler {
    passage_index: HashMap<SmolStr, usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            passage_index: HashMap::new(),
        }
    }

    pub fn compile(&mut self, scene_file: SceneFile) -> CompileResult<Scene> {
        if scene_file.passages.is_empty() {
            return Err(CompileError::EmptyScene {
                span: scene_file.span,
            });
        }

        self.build_passage_index(&scene_file.passages)?;

        let passages = scene_file
            .passages
            .into_iter()
            .map(|p| self.compile_passage(p))
            .collect::<CompileResult<Vec<_>>>()?;

        let fm = scene_file.frontmatter;

        let context = match fm.context {
            ast::Context::Journey => Some(ContextId::new("journey")),
            ast::Context::Port => Some(ContextId::new("port")),
            ast::Context::Any => None,
        };

        let requirements = fm
            .requires
            .as_ref()
            .map(|r| self.compile_requirements(r))
            .unwrap_or(Requirement::Always);

        // Generate Rhai for scene requirements (if any)
        let rhai_requirements = fm.requires.as_ref().map(|r| {
            let condition = Condition {
                clauses: self.requirements_to_clauses(r),
                span: r.span.clone(),
            };
            SmolStr::new(codegen::generate_condition(&condition))
        });

        let tags = fm
            .tags
            .into_iter()
            .map(|t| TagId::new(t))
            .collect::<Tags>();

        Ok(Scene {
            id: SceneId::new(fm.id),
            title: fm.title,
            tags,
            context,
            weight: fm.weight,
            cooldown: fm.cooldown as u64,
            requirements,
            passages,
            rhai_requirements,
        })
    }

    fn build_passage_index(&mut self, passages: &[Passage]) -> CompileResult<()> {
        self.passage_index.clear();

        for (index, passage) in passages.iter().enumerate() {
            if self.passage_index.contains_key(&passage.name) {
                return Err(CompileError::DuplicatePassage {
                    name: passage.name.to_string(),
                    span: passage.span.clone(),
                });
            }
            self.passage_index.insert(passage.name.clone(), index);
        }

        Ok(())
    }

    fn compile_passage(&self, passage: Passage) -> CompileResult<RuntimePassage> {
        let mut text_parts = Vec::new();
        let mut choices = Vec::new();
        let mut passage_effects = Vec::new();
        let mut rhai_scripts = Vec::new();

        for content in passage.content {
            match content {
                PassageContent::Prose(p) => {
                    text_parts.push(p.text.to_string());
                }
                PassageContent::Choice(choice) => {
                    choices.push(self.compile_choice(choice)?);
                }
                PassageContent::RhaiBlock(block) => {
                    // Store the Rhai source for the passage
                    rhai_scripts.push(block.source.to_string());
                    // Also create Effect::Script for runtime execution
                    passage_effects.push(Effect::Script {
                        source: block.source,
                    });
                }
            }
        }

        let text = text_parts.join("\n");

        // Combine Rhai scripts into rhai_on_enter
        let rhai_on_enter = if rhai_scripts.is_empty() {
            None
        } else {
            Some(SmolStr::new(rhai_scripts.join("\n")))
        };

        // If there are passage-level effects and choices, prepend effects to first choice
        // or create a synthetic "Continue" choice if no choices exist
        if !passage_effects.is_empty() {
            if choices.is_empty() {
                // Create a synthetic continue choice with the effects
                choices.push(RuntimeChoice {
                    text: SmolStr::new("Continue"),
                    requirements: Requirement::Always,
                    effects: passage_effects,
                    next: Navigation::End,
                    rhai_condition: None,
                    rhai_effects: rhai_on_enter.clone(),
                });
            } else {
                // Prepend effects to the first choice
                let mut first_choice = choices.remove(0);
                let mut combined_effects = passage_effects;
                combined_effects.extend(first_choice.effects);
                first_choice.effects = combined_effects;
                // Also prepend Rhai to first choice's rhai_effects
                if let Some(ref passage_rhai) = rhai_on_enter {
                    let combined_rhai = if let Some(ref choice_rhai) = first_choice.rhai_effects {
                        format!("{}\n{}", passage_rhai, choice_rhai)
                    } else {
                        passage_rhai.to_string()
                    };
                    first_choice.rhai_effects = Some(SmolStr::new(combined_rhai));
                }
                choices.insert(0, first_choice);
            }
        }

        Ok(RuntimePassage {
            text: text.into(),
            choices,
            rhai_on_enter,
        })
    }

    fn compile_choice(&self, choice: ast::Choice) -> CompileResult<RuntimeChoice> {
        // Generate Rhai condition if present
        let rhai_condition = choice
            .condition
            .as_ref()
            .map(|c| SmolStr::new(codegen::generate_condition(c)));

        // Generate Rhai effects
        let rhai_effects = if choice.effects.is_empty() {
            None
        } else {
            Some(SmolStr::new(codegen::generate_effects(&choice.effects)))
        };

        // Also compile to native Requirement/Effect for backward compatibility
        let requirements = choice
            .condition
            .as_ref()
            .map(|c| self.compile_condition(c))
            .unwrap_or(Requirement::Always);

        let effects = choice
            .effects
            .iter()
            .map(|e| self.compile_effect(e))
            .collect();

        let next = match choice.target {
            Some(target) if target.is_end => Navigation::End,
            Some(target) => {
                let index = self
                    .passage_index
                    .get(&target.target)
                    .copied()
                    .ok_or_else(|| CompileError::UnknownPassage {
                        name: target.target.to_string(),
                        span: target.span.clone(),
                    })?;
                Navigation::Passage(index)
            }
            None => Navigation::End,
        };

        Ok(RuntimeChoice {
            text: choice.text,
            requirements,
            effects,
            next,
            rhai_condition,
            rhai_effects,
        })
    }

    fn compile_condition(&self, condition: &Condition) -> Requirement {
        let reqs: Vec<Requirement> = condition
            .clauses
            .iter()
            .map(|clause| self.compile_clause(clause))
            .collect();

        Requirement::and(reqs)
    }

    fn compile_clause(&self, clause: &ConditionClause) -> Requirement {
        match clause {
            ConditionClause::Tag(tag_cond) => {
                let category = match tag_cond.source {
                    TagSource::Crew => TagCategoryId::new("crew"),
                    TagSource::Ship => TagCategoryId::new("ship"),
                    TagSource::Cargo => TagCategoryId::new("cargo"),
                };
                Requirement::HasTag {
                    category,
                    tag: TagId::new(tag_cond.tag.clone()),
                }
            }
            ConditionClause::Resource(res_cond) => {
                let resource = ResourceId::new(res_cond.resource.clone());
                match res_cond.operator {
                    CompareOp::Ge => Requirement::MinResource {
                        resource,
                        value: res_cond.value,
                    },
                    CompareOp::Gt => Requirement::MinResource {
                        resource,
                        value: res_cond.value + 1,
                    },
                    CompareOp::Le => Requirement::MaxResource {
                        resource,
                        value: res_cond.value,
                    },
                    CompareOp::Lt => Requirement::MaxResource {
                        resource,
                        value: res_cond.value - 1,
                    },
                    CompareOp::Eq => Requirement::And(vec![
                        Requirement::MinResource {
                            resource: resource.clone(),
                            value: res_cond.value,
                        },
                        Requirement::MaxResource {
                            resource,
                            value: res_cond.value,
                        },
                    ]),
                    CompareOp::Ne => Requirement::Or(vec![
                        Requirement::MinResource {
                            resource: resource.clone(),
                            value: res_cond.value + 1,
                        },
                        Requirement::MaxResource {
                            resource,
                            value: res_cond.value - 1,
                        },
                    ]),
                }
            }
            ConditionClause::Flag(flag_cond) => self.compile_flag_condition(flag_cond),
        }
    }

    fn compile_flag_condition(&self, cond: &FlagCondition) -> Requirement {
        let flag = FlagId::new(cond.flag.clone());

        let base_req = match (&cond.operator, &cond.value) {
            (None, None) => Requirement::HasFlag { flag },
            (None, Some(FlagValue::Bool(b))) => {
                if *b {
                    Requirement::HasFlag { flag }
                } else {
                    Requirement::NotFlag { flag }
                }
            }
            (Some(op), Some(FlagValue::Int(v))) => {
                let core_op = match op {
                    CompareOp::Eq => engine_core::CompareOp::Eq,
                    CompareOp::Ne => engine_core::CompareOp::Ne,
                    CompareOp::Lt => engine_core::CompareOp::Lt,
                    CompareOp::Le => engine_core::CompareOp::Le,
                    CompareOp::Gt => engine_core::CompareOp::Gt,
                    CompareOp::Ge => engine_core::CompareOp::Ge,
                };
                Requirement::FlagCompare {
                    flag,
                    op: core_op,
                    value: *v,
                }
            }
            (Some(_), Some(FlagValue::String(s))) => Requirement::FlagEquals {
                flag,
                value: Value::String(s.clone()),
            },
            (Some(_), Some(FlagValue::Bool(b))) => Requirement::FlagEquals {
                flag,
                value: Value::Bool(*b),
            },
            _ => Requirement::HasFlag { flag },
        };

        if cond.negated {
            Requirement::not(base_req)
        } else {
            base_req
        }
    }

    fn compile_effect(&self, effect: &ast::Effect) -> Effect {
        match effect {
            ast::Effect::Resource(res) => {
                let resource = ResourceId::new(res.resource.clone());
                match res.operator {
                    AssignOp::Add => Effect::ModifyResource {
                        resource,
                        delta: res.value,
                    },
                    AssignOp::Sub => Effect::ModifyResource {
                        resource,
                        delta: -res.value,
                    },
                    AssignOp::Set => Effect::SetResource {
                        resource,
                        value: res.value,
                    },
                }
            }
            ast::Effect::Flag(flag) => {
                let flag_id = FlagId::new(flag.flag.clone());
                let value = match &flag.value {
                    FlagValue::Bool(b) => Value::Bool(*b),
                    FlagValue::Int(n) => Value::Int(*n),
                    FlagValue::String(s) => Value::String(s.clone()),
                };
                Effect::SetFlag {
                    flag: flag_id,
                    value,
                }
            }
            ast::Effect::AddCard(add) => Effect::AddCard {
                card_id: engine_core::CardId::new(add.card_id.clone()),
            },
            ast::Effect::RemoveCards(remove) => Effect::RemoveCards {
                pattern: remove.pattern.clone(),
            },
            ast::Effect::Chronicle(chron) => Effect::AddChronicle {
                title: chron.title.clone(),
                text: chron.text.clone(),
            },
            ast::Effect::Damage(dmg) => Effect::Damage {
                resource: ResourceId::new(dmg.target.clone()),
                amount: dmg.value,
            },
            ast::Effect::Reputation(rep) => {
                let faction = engine_core::FactionId::new(rep.faction.clone());
                let delta = match rep.operator {
                    AssignOp::Add => rep.value,
                    AssignOp::Sub => -rep.value,
                    AssignOp::Set => rep.value, // For set, we'd need a different effect type
                };
                Effect::ModifyFactionReputation { faction, delta }
            }
            ast::Effect::Script(script) => Effect::Script {
                source: script.source.clone(),
            },
        }
    }

    fn compile_requirements(&self, reqs: &ast::Requirements) -> Requirement {
        let mut clauses = Vec::new();

        for tag in &reqs.ship_tags {
            clauses.push(Requirement::HasTag {
                category: TagCategoryId::new("ship"),
                tag: TagId::new(tag.clone()),
            });
        }

        for tag in &reqs.crew_tags {
            clauses.push(Requirement::HasTag {
                category: TagCategoryId::new("crew"),
                tag: TagId::new(tag.clone()),
            });
        }

        for tag in &reqs.cargo_tags {
            clauses.push(Requirement::HasTag {
                category: TagCategoryId::new("cargo"),
                tag: TagId::new(tag.clone()),
            });
        }

        for res in &reqs.min_resources {
            clauses.push(Requirement::MinResource {
                resource: ResourceId::new(res.resource.clone()),
                value: res.value,
            });
        }

        for res in &reqs.max_resources {
            clauses.push(Requirement::MaxResource {
                resource: ResourceId::new(res.resource.clone()),
                value: res.value,
            });
        }

        for flag in &reqs.required_flags {
            clauses.push(Requirement::HasFlag {
                flag: FlagId::new(flag.clone()),
            });
        }

        for flag in &reqs.excluded_flags {
            clauses.push(Requirement::NotFlag {
                flag: FlagId::new(flag.clone()),
            });
        }

        Requirement::and(clauses)
    }

    /// Convert Requirements (from frontmatter) to ConditionClauses for Rhai codegen.
    fn requirements_to_clauses(&self, reqs: &ast::Requirements) -> Vec<ConditionClause> {
        let mut clauses = Vec::new();

        for tag in &reqs.ship_tags {
            clauses.push(ConditionClause::Tag(ast::TagCondition {
                source: TagSource::Ship,
                tag: tag.clone(),
                span: reqs.span.clone(),
            }));
        }

        for tag in &reqs.crew_tags {
            clauses.push(ConditionClause::Tag(ast::TagCondition {
                source: TagSource::Crew,
                tag: tag.clone(),
                span: reqs.span.clone(),
            }));
        }

        for tag in &reqs.cargo_tags {
            clauses.push(ConditionClause::Tag(ast::TagCondition {
                source: TagSource::Cargo,
                tag: tag.clone(),
                span: reqs.span.clone(),
            }));
        }

        for res in &reqs.min_resources {
            clauses.push(ConditionClause::Resource(ast::ResourceCondition {
                resource: res.resource.clone(),
                operator: CompareOp::Ge,
                value: res.value,
                span: reqs.span.clone(),
            }));
        }

        for res in &reqs.max_resources {
            clauses.push(ConditionClause::Resource(ast::ResourceCondition {
                resource: res.resource.clone(),
                operator: CompareOp::Le,
                value: res.value,
                span: reqs.span.clone(),
            }));
        }

        for flag in &reqs.required_flags {
            clauses.push(ConditionClause::Flag(FlagCondition {
                flag: flag.clone(),
                negated: false,
                operator: None,
                value: None,
                span: reqs.span.clone(),
            }));
        }

        for flag in &reqs.excluded_flags {
            clauses.push(ConditionClause::Flag(FlagCondition {
                flag: flag.clone(),
                negated: true,
                operator: None,
                value: None,
                span: reqs.span.clone(),
            }));
        }

        clauses
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

pub fn compile(scene_file: SceneFile) -> CompileResult<Scene> {
    Compiler::new().compile(scene_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_scene() -> SceneFile {
        use holdsmith_parser::*;

        SceneFile {
            frontmatter: Frontmatter {
                id: "test_scene".into(),
                title: "Test Scene".into(),
                tags: vec!["danger".into()],
                context: Context::Journey,
                weight: 10,
                cooldown: 5,
                requires: None,
                span: 0..100,
            },
            passages: vec![
                Passage {
                    name: "intro".into(),
                    content: vec![
                        PassageContent::Prose(Prose {
                            text: "You see something.".into(),
                            span: 0..20,
                        }),
                        PassageContent::Choice(Choice {
                            text: "Investigate".into(),
                            condition: None,
                            effects: vec![],
                            target: Some(NavigationTarget {
                                target: "investigate".into(),
                                is_end: false,
                                span: 30..45,
                            }),
                            span: 20..50,
                        }),
                        PassageContent::Choice(Choice {
                            text: "Leave".into(),
                            condition: None,
                            effects: vec![],
                            target: Some(NavigationTarget {
                                target: "END".into(),
                                is_end: true,
                                span: 55..60,
                            }),
                            span: 50..65,
                        }),
                    ],
                    span: 0..70,
                },
                Passage {
                    name: "investigate".into(),
                    content: vec![
                        PassageContent::Prose(Prose {
                            text: "You found treasure!".into(),
                            span: 70..90,
                        }),
                        PassageContent::Choice(Choice {
                            text: "Take it".into(),
                            condition: None,
                            effects: vec![ast::Effect::Resource(ResourceEffect {
                                resource: "credits".into(),
                                operator: AssignOp::Add,
                                value: 50,
                                span: 95..110,
                            })],
                            target: Some(NavigationTarget {
                                target: "END".into(),
                                is_end: true,
                                span: 110..115,
                            }),
                            span: 90..120,
                        }),
                    ],
                    span: 70..125,
                },
            ],
            span: 0..125,
        }
    }

    #[test]
    fn test_compile_simple_scene() {
        let scene_file = make_simple_scene();
        let scene = compile(scene_file).expect("compilation should succeed");

        assert_eq!(scene.id.as_str(), "test_scene");
        assert_eq!(scene.title.as_str(), "Test Scene");
        assert_eq!(scene.weight, 10);
        assert_eq!(scene.cooldown, 5);
        assert_eq!(scene.passages.len(), 2);
        assert_eq!(scene.context, Some(ContextId::new("journey")));
    }

    #[test]
    fn test_passage_navigation() {
        let scene_file = make_simple_scene();
        let scene = compile(scene_file).expect("compilation should succeed");

        let intro = &scene.passages[0];
        assert_eq!(intro.choices.len(), 2);
        assert!(matches!(intro.choices[0].next, Navigation::Passage(1)));
        assert!(matches!(intro.choices[1].next, Navigation::End));
    }

    #[test]
    fn test_effect_compilation() {
        let scene_file = make_simple_scene();
        let scene = compile(scene_file).expect("compilation should succeed");

        let investigate = &scene.passages[1];
        let take_choice = &investigate.choices[0];
        assert_eq!(take_choice.effects.len(), 1);

        match &take_choice.effects[0] {
            Effect::ModifyResource { resource, delta } => {
                assert_eq!(resource.as_str(), "credits");
                assert_eq!(*delta, 50);
            }
            _ => panic!("expected ModifyResource effect"),
        }
    }

    #[test]
    fn test_unknown_passage_error() {
        use holdsmith_parser::*;

        let scene_file = SceneFile {
            frontmatter: Frontmatter {
                id: "test".into(),
                title: "Test".into(),
                tags: vec![],
                context: Context::Any,
                weight: 1,
                cooldown: 0,
                requires: None,
                span: 0..10,
            },
            passages: vec![Passage {
                name: "intro".into(),
                content: vec![PassageContent::Choice(Choice {
                    text: "Go".into(),
                    condition: None,
                    effects: vec![],
                    target: Some(NavigationTarget {
                        target: "nonexistent".into(),
                        is_end: false,
                        span: 20..35,
                    }),
                    span: 15..40,
                })],
                span: 10..45,
            }],
            span: 0..50,
        };

        let result = compile(scene_file);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CompileError::UnknownPassage { .. }
        ));
    }
}
