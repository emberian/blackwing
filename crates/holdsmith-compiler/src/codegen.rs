//! Rhai code generation from Holdsmith DSL constructs.
//!
//! This module generates Rhai source code from parsed DSL conditions and effects,
//! creating a unified semantic model where all game logic compiles to Rhai.
//!
//! ## Why Rhai Codegen?
//!
//! 1. **Unified Analysis**: The analyzer only needs to understand Rhai semantics
//! 2. **Single Source of Truth**: DSL syntax becomes sugar over Rhai
//! 3. **Full Expressiveness**: Complex logic can use Rhai directly via `{ }` blocks

use holdsmith_parser::{
    AssignOp, CompareOp, Condition, ConditionClause, Effect as AstEffect, FlagCondition,
    FlagValue, ResourceCondition, TagCondition, TagSource,
};

/// Generate Rhai source for a condition (returns boolean expression).
///
/// # Example DSL → Rhai
/// - `{ credits >= 100 }` → `resource("credits") >= 100`
/// - `{ crew.engineering }` → `has_tag("crew", "engineering")`
/// - `{ !visited_port }` → `!flag_bool("visited_port")`
pub fn generate_condition(condition: &Condition) -> String {
    if condition.clauses.is_empty() {
        return "true".to_string();
    }

    let clauses: Vec<String> = condition
        .clauses
        .iter()
        .map(generate_clause)
        .collect();

    // Join with && (all conditions must be true)
    clauses.join(" && ")
}

/// Generate Rhai source for a single condition clause.
fn generate_clause(clause: &ConditionClause) -> String {
    match clause {
        ConditionClause::Tag(tag) => generate_tag_condition(tag),
        ConditionClause::Resource(res) => generate_resource_condition(res),
        ConditionClause::Flag(flag) => generate_flag_condition(flag),
    }
}

fn generate_tag_condition(cond: &TagCondition) -> String {
    let category = match cond.source {
        TagSource::Crew => "crew",
        TagSource::Ship => "ship",
        TagSource::Cargo => "cargo",
    };
    format!(r#"has_tag("{}", "{}")"#, category, cond.tag)
}

fn generate_resource_condition(cond: &ResourceCondition) -> String {
    let op = match cond.operator {
        CompareOp::Ge => ">=",
        CompareOp::Le => "<=",
        CompareOp::Gt => ">",
        CompareOp::Lt => "<",
        CompareOp::Eq => "==",
        CompareOp::Ne => "!=",
    };
    format!(r#"resource("{}") {} {}"#, cond.resource, op, cond.value)
}

fn generate_flag_condition(cond: &FlagCondition) -> String {
    let base = match (&cond.operator, &cond.value) {
        // Simple flag existence check
        (None, None) => format!(r#"flag_bool("{}")"#, cond.flag),

        // Flag equals boolean
        (None, Some(FlagValue::Bool(true))) => format!(r#"flag_bool("{}")"#, cond.flag),
        (None, Some(FlagValue::Bool(false))) => format!(r#"!flag_bool("{}")"#, cond.flag),

        // Flag comparison with int
        (Some(op), Some(FlagValue::Int(v))) => {
            let op_str = match op {
                CompareOp::Ge => ">=",
                CompareOp::Le => "<=",
                CompareOp::Gt => ">",
                CompareOp::Lt => "<",
                CompareOp::Eq => "==",
                CompareOp::Ne => "!=",
            };
            format!(r#"flag("{}") {} {}"#, cond.flag, op_str, v)
        }

        // Flag comparison with string
        (Some(_), Some(FlagValue::String(s))) => {
            format!(r#"flag("{}") == "{}""#, cond.flag, s)
        }

        // Flag comparison with bool (using ==)
        (Some(_), Some(FlagValue::Bool(b))) => {
            format!(r#"flag("{}") == {}"#, cond.flag, b)
        }

        // Default: simple existence check
        _ => format!(r#"flag_bool("{}")"#, cond.flag),
    };

    if cond.negated {
        format!("!({})", base)
    } else {
        base
    }
}

/// Generate Rhai source for a list of effects.
///
/// # Example DSL → Rhai
/// - `~ credits += 50` → `modify_resource("credits", 50);`
/// - `~ damage hull 10` → `damage("hull", 10);`
/// - `~ flag visited = true` → `set_flag("visited", true);`
/// - `~ chronicle "Title"` → `chronicle("Title", "");`
pub fn generate_effects(effects: &[AstEffect]) -> String {
    effects
        .iter()
        .map(generate_effect)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate Rhai source for a single effect.
fn generate_effect(effect: &AstEffect) -> String {
    match effect {
        AstEffect::Resource(res) => {
            match res.operator {
                AssignOp::Add => format!(r#"modify_resource("{}", {});"#, res.resource, res.value),
                AssignOp::Sub => format!(r#"modify_resource("{}", {});"#, res.resource, -res.value),
                AssignOp::Set => format!(r#"set_resource("{}", {});"#, res.resource, res.value),
            }
        }

        AstEffect::Flag(flag) => {
            let value_str = match &flag.value {
                FlagValue::Bool(b) => format!("{}", b),
                FlagValue::Int(n) => format!("{}", n),
                FlagValue::String(s) => format!(r#""{}""#, s),
            };
            format!(r#"set_flag("{}", {});"#, flag.flag, value_str)
        }

        AstEffect::AddCard(add) => {
            format!(r#"add_card("{}");"#, add.card_id)
        }

        AstEffect::RemoveCards(remove) => {
            format!(r#"remove_cards("{}");"#, remove.pattern)
        }

        AstEffect::Chronicle(chron) => {
            // Escape any quotes in the text
            let title = chron.title.replace('"', r#"\""#);
            let text = chron.text.replace('"', r#"\""#);
            format!(r#"chronicle("{}", "{}");"#, title, text)
        }

        AstEffect::Damage(dmg) => {
            format!(r#"damage("{}", {});"#, dmg.target, dmg.value)
        }

        AstEffect::Reputation(rep) => {
            let delta = match rep.operator {
                AssignOp::Add => rep.value,
                AssignOp::Sub => -rep.value,
                AssignOp::Set => rep.value, // Note: set not really supported, treated as delta
            };
            format!(r#"modify_reputation("{}", {});"#, rep.faction, delta)
        }

        AstEffect::Script(script) => {
            // Script effects are already Rhai - include directly
            script.source.to_string()
        }
    }
}

/// Generate a complete Rhai script for a choice (condition check + effects).
///
/// Returns Rhai code that:
/// 1. Checks the condition (if any)
/// 2. Executes effects if condition passes
/// 3. Optionally navigates via jump_to()
pub fn generate_choice_script(
    condition: Option<&Condition>,
    effects: &[AstEffect],
    navigation: Option<&str>,
) -> String {
    let mut parts = Vec::new();

    // Generate condition check
    let condition_code = condition
        .map(generate_condition)
        .unwrap_or_else(|| "true".to_string());

    // Generate effects
    let effects_code = generate_effects(effects);

    // Generate navigation
    let nav_code = navigation
        .map(|target| {
            if target == "END" {
                String::new() // No jump_to for END
            } else {
                format!(r#"jump_to("{}");"#, target)
            }
        })
        .unwrap_or_default();

    // Combine into a script
    if condition.is_some() {
        parts.push(format!("if {} {{", condition_code));
        if !effects_code.is_empty() {
            for line in effects_code.lines() {
                parts.push(format!("    {}", line));
            }
        }
        if !nav_code.is_empty() {
            parts.push(format!("    {}", nav_code));
        }
        parts.push("}".to_string());
    } else {
        if !effects_code.is_empty() {
            parts.push(effects_code);
        }
        if !nav_code.is_empty() {
            parts.push(nav_code);
        }
    }

    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use holdsmith_parser::{
        AddCardEffect, ChronicleEffect, DamageEffect, FlagEffect, ReputationEffect,
        ResourceEffect, ScriptEffect,
    };
    use smol_str::SmolStr;

    fn make_span() -> std::ops::Range<usize> {
        0..0
    }

    #[test]
    fn test_generate_resource_condition() {
        let cond = ResourceCondition {
            resource: SmolStr::new("credits"),
            operator: CompareOp::Ge,
            value: 100,
            span: make_span(),
        };
        let clause = ConditionClause::Resource(cond);
        assert_eq!(generate_clause(&clause), r#"resource("credits") >= 100"#);
    }

    #[test]
    fn test_generate_tag_condition() {
        let cond = TagCondition {
            source: TagSource::Crew,
            tag: SmolStr::new("engineering"),
            span: make_span(),
        };
        let clause = ConditionClause::Tag(cond);
        assert_eq!(generate_clause(&clause), r#"has_tag("crew", "engineering")"#);
    }

    #[test]
    fn test_generate_flag_condition_simple() {
        let cond = FlagCondition {
            flag: SmolStr::new("visited_port"),
            negated: false,
            operator: None,
            value: None,
            span: make_span(),
        };
        let clause = ConditionClause::Flag(cond);
        assert_eq!(generate_clause(&clause), r#"flag_bool("visited_port")"#);
    }

    #[test]
    fn test_generate_flag_condition_negated() {
        let cond = FlagCondition {
            flag: SmolStr::new("visited_port"),
            negated: true,
            operator: None,
            value: None,
            span: make_span(),
        };
        let clause = ConditionClause::Flag(cond);
        assert_eq!(generate_clause(&clause), r#"!(flag_bool("visited_port"))"#);
    }

    #[test]
    fn test_generate_flag_condition_comparison() {
        let cond = FlagCondition {
            flag: SmolStr::new("visit_count"),
            negated: false,
            operator: Some(CompareOp::Ge),
            value: Some(FlagValue::Int(3)),
            span: make_span(),
        };
        let clause = ConditionClause::Flag(cond);
        assert_eq!(generate_clause(&clause), r#"flag("visit_count") >= 3"#);
    }

    #[test]
    fn test_generate_combined_condition() {
        let condition = Condition {
            clauses: vec![
                ConditionClause::Resource(ResourceCondition {
                    resource: SmolStr::new("credits"),
                    operator: CompareOp::Ge,
                    value: 100,
                    span: make_span(),
                }),
                ConditionClause::Tag(TagCondition {
                    source: TagSource::Ship,
                    tag: SmolStr::new("combat"),
                    span: make_span(),
                }),
            ],
            span: make_span(),
        };
        assert_eq!(
            generate_condition(&condition),
            r#"resource("credits") >= 100 && has_tag("ship", "combat")"#
        );
    }

    #[test]
    fn test_generate_resource_effect_add() {
        let effect = AstEffect::Resource(ResourceEffect {
            resource: SmolStr::new("credits"),
            operator: AssignOp::Add,
            value: 50,
            span: make_span(),
        });
        assert_eq!(generate_effect(&effect), r#"modify_resource("credits", 50);"#);
    }

    #[test]
    fn test_generate_resource_effect_sub() {
        let effect = AstEffect::Resource(ResourceEffect {
            resource: SmolStr::new("credits"),
            operator: AssignOp::Sub,
            value: 25,
            span: make_span(),
        });
        assert_eq!(generate_effect(&effect), r#"modify_resource("credits", -25);"#);
    }

    #[test]
    fn test_generate_damage_effect() {
        let effect = AstEffect::Damage(DamageEffect {
            target: SmolStr::new("hull"),
            value: 10,
            span: make_span(),
        });
        assert_eq!(generate_effect(&effect), r#"damage("hull", 10);"#);
    }

    #[test]
    fn test_generate_flag_effect() {
        let effect = AstEffect::Flag(FlagEffect {
            flag: SmolStr::new("visited"),
            value: FlagValue::Bool(true),
            span: make_span(),
        });
        assert_eq!(generate_effect(&effect), r#"set_flag("visited", true);"#);
    }

    #[test]
    fn test_generate_add_card_effect() {
        let effect = AstEffect::AddCard(AddCardEffect {
            card_id: SmolStr::new("cargo_gold"),
            span: make_span(),
        });
        assert_eq!(generate_effect(&effect), r#"add_card("cargo_gold");"#);
    }

    #[test]
    fn test_generate_chronicle_effect() {
        let effect = AstEffect::Chronicle(ChronicleEffect {
            title: SmolStr::new("Great Discovery"),
            text: SmolStr::new("Found ancient ruins"),
            span: make_span(),
        });
        assert_eq!(
            generate_effect(&effect),
            r#"chronicle("Great Discovery", "Found ancient ruins");"#
        );
    }

    #[test]
    fn test_generate_reputation_effect() {
        let effect = AstEffect::Reputation(ReputationEffect {
            faction: SmolStr::new("merchants"),
            operator: AssignOp::Add,
            value: 10,
            span: make_span(),
        });
        assert_eq!(generate_effect(&effect), r#"modify_reputation("merchants", 10);"#);
    }

    #[test]
    fn test_generate_script_effect_passthrough() {
        let effect = AstEffect::Script(ScriptEffect {
            source: SmolStr::new("if rng_float() < 0.5 { damage(\"hull\", 5); }"),
            span: make_span(),
        });
        assert_eq!(
            generate_effect(&effect),
            r#"if rng_float() < 0.5 { damage("hull", 5); }"#
        );
    }

    #[test]
    fn test_generate_multiple_effects() {
        let effects = vec![
            AstEffect::Resource(ResourceEffect {
                resource: SmolStr::new("credits"),
                operator: AssignOp::Add,
                value: 50,
                span: make_span(),
            }),
            AstEffect::Damage(DamageEffect {
                target: SmolStr::new("hull"),
                value: 10,
                span: make_span(),
            }),
        ];
        let code = generate_effects(&effects);
        assert!(code.contains(r#"modify_resource("credits", 50);"#));
        assert!(code.contains(r#"damage("hull", 10);"#));
    }

    #[test]
    fn test_generate_choice_script_with_condition() {
        let condition = Condition {
            clauses: vec![ConditionClause::Resource(ResourceCondition {
                resource: SmolStr::new("credits"),
                operator: CompareOp::Ge,
                value: 100,
                span: make_span(),
            })],
            span: make_span(),
        };
        let effects = vec![AstEffect::Resource(ResourceEffect {
            resource: SmolStr::new("credits"),
            operator: AssignOp::Sub,
            value: 50,
            span: make_span(),
        })];

        let script = generate_choice_script(Some(&condition), &effects, Some("next"));
        assert!(script.contains(r#"if resource("credits") >= 100 {"#));
        assert!(script.contains(r#"modify_resource("credits", -50);"#));
        assert!(script.contains(r#"jump_to("next");"#));
    }
}
