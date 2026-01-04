mod compiler;
mod error;
mod validator;

pub use compiler::{compile, Compiler};
pub use error::{CompileError, CompileResult, ValidationError, ValidationReport, ValidationWarning};
pub use validator::{validate, validate_with_cards, Validator};

use engine_core::{GameSchema, Scene};

pub struct CompilationPipeline<'a> {
    schema: &'a GameSchema,
}

impl<'a> CompilationPipeline<'a> {
    pub fn new(schema: &'a GameSchema) -> Self {
        Self { schema }
    }

    pub fn compile_and_validate(
        &self,
        source: &str,
    ) -> Result<(Scene, ValidationReport), PipelineError> {
        self.compile_and_validate_named(source, "<input>")
    }

    pub fn compile_and_validate_named(
        &self,
        source: &str,
        filename: &str,
    ) -> Result<(Scene, ValidationReport), PipelineError> {
        let ast = holdsmith_parser::parse(source, filename).map_err(PipelineError::Parse)?;

        let scene = compile(ast).map_err(PipelineError::Compile)?;

        let report = validate(&scene, self.schema);

        Ok((scene, report))
    }

    pub fn compile_all(
        &self,
        sources: impl IntoIterator<Item = (String, String)>,
    ) -> CompilationResult {
        let mut scenes = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for (name, source) in sources {
            match self.compile_and_validate_named(&source, &name) {
                Ok((scene, report)) => {
                    if report.is_ok() {
                        warnings.extend(
                            report
                                .warnings
                                .into_iter()
                                .map(|w| (name.clone(), w)),
                        );
                        scenes.push(scene);
                    } else {
                        errors.extend(
                            report
                                .errors
                                .into_iter()
                                .map(|e| (name.clone(), PipelineError::Validation(e))),
                        );
                    }
                }
                Err(e) => {
                    errors.push((name, e));
                }
            }
        }

        CompilationResult {
            scenes,
            errors,
            warnings,
        }
    }
}

pub struct CompilationResult {
    pub scenes: Vec<Scene>,
    pub errors: Vec<(String, PipelineError)>,
    pub warnings: Vec<(String, ValidationWarning)>,
}

impl CompilationResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("parse error: {0}")]
    Parse(#[from] holdsmith_parser::ParseError),

    #[error("compile error: {0}")]
    Compile(#[from] CompileError),

    #[error("validation error: {0}")]
    Validation(ValidationError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::*;

    fn test_schema() -> GameSchema {
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
                ResourceDef {
                    id: ResourceId::new("integrity"),
                    name: "Integrity".to_string(),
                    min: Some(0),
                    max: Some(100),
                    default: 75,
                    on_zero: Some(OnZeroBehavior::GameOver {
                        reason: "rampancy".to_string(),
                    }),
                },
            ],
            tag_categories: vec![
                TagCategory {
                    id: TagCategoryId::new("ship"),
                    name: "Ship".to_string(),
                    tags: vec![
                        TagId::new("combat"),
                        TagId::new("stealth"),
                        TagId::new("sensor"),
                    ],
                },
                TagCategory {
                    id: TagCategoryId::new("crew"),
                    name: "Crew".to_string(),
                    tags: vec![TagId::new("engineering"), TagId::new("medical")],
                },
            ],
            card_types: vec![],
            contexts: vec![
                ContextDef {
                    id: ContextId::new("journey"),
                    name: "Journey".to_string(),
                },
                ContextDef {
                    id: ContextId::new("port"),
                    name: "Port".to_string(),
                },
            ],
            slot_types: vec![],
            location_statuses: vec![],
            initial_state: InitialStateDef::default(),
        }
    }

    const SIMPLE_SCENE: &str = r#"---
id: test_encounter
title: Test Encounter
tags: [danger]
context: journey
weight: 10
cooldown: 5
---

=== intro
Something appears on your sensors.

* [Investigate]
  -> investigate
* [Ignore it]
  -> END

=== investigate
You found credits!

* [Take them]
  ~ credits += 50
  -> END
"#;

    #[test]
    fn test_full_pipeline() {
        let schema = test_schema();
        let pipeline = CompilationPipeline::new(&schema);

        let (scene, report) = pipeline.compile_and_validate(SIMPLE_SCENE).unwrap();

        assert!(report.is_ok());
        assert_eq!(scene.id.as_str(), "test_encounter");
        assert_eq!(scene.passages.len(), 2);
    }

    #[test]
    fn test_compile_all() {
        let schema = test_schema();
        let pipeline = CompilationPipeline::new(&schema);

        let sources = vec![
            ("scene1.scene".to_string(), SIMPLE_SCENE.to_string()),
            ("scene2.scene".to_string(), SIMPLE_SCENE.replace("test_encounter", "test_encounter_2")),
        ];

        let result = pipeline.compile_all(sources);

        assert!(result.is_ok());
        assert_eq!(result.scenes.len(), 2);
    }
}
