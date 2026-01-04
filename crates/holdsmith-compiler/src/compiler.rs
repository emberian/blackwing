use std::collections::HashMap;

use engine_core::{
    Choice as RuntimeChoice, ContextId, Navigation, Passage as RuntimePassage, Scene, SceneId,
    TagId, Tags,
};
use holdsmith_parser::{
    self as ast, CompareOp, Condition, ConditionClause, FlagCondition, Passage, PassageContent,
    SceneFile, TagSource,
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

        // Generate Rhai for scene requirements
        let rhai_requirements = fm
            .requires
            .as_ref()
            .map(|r| {
                let condition = Condition {
                    clauses: self.requirements_to_clauses(r),
                    span: r.span.clone(),
                };
                SmolStr::new(codegen::generate_condition(&condition))
            })
            .unwrap_or_default();

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
                    rhai_scripts.push(block.source.to_string());
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

        // If there are passage-level Rhai scripts and no choices, create a Continue choice
        if rhai_on_enter.is_some() && choices.is_empty() {
            choices.push(RuntimeChoice {
                text: SmolStr::new("Continue"),
                next: Navigation::End,
                rhai_condition: SmolStr::default(),
                rhai_effects: rhai_on_enter.clone().unwrap_or_default(),
            });
        }

        Ok(RuntimePassage {
            text: text.into(),
            choices,
            rhai_on_enter,
        })
    }

    fn compile_choice(&self, choice: ast::Choice) -> CompileResult<RuntimeChoice> {
        // Generate Rhai condition
        let rhai_condition = choice
            .condition
            .as_ref()
            .map(|c| SmolStr::new(codegen::generate_condition(c)))
            .unwrap_or_default();

        // Generate Rhai effects
        let rhai_effects = if choice.effects.is_empty() {
            SmolStr::default()
        } else {
            SmolStr::new(codegen::generate_effects(&choice.effects))
        };

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
            next,
            rhai_condition,
            rhai_effects,
        })
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
    fn test_rhai_effects_generated() {
        let scene_file = make_simple_scene();
        let scene = compile(scene_file).expect("compilation should succeed");

        let investigate = &scene.passages[1];
        let take_choice = &investigate.choices[0];

        // Effects are now Rhai, not native
        assert!(!take_choice.rhai_effects.is_empty());
        assert!(take_choice.rhai_effects.contains("modify_resource"));
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
