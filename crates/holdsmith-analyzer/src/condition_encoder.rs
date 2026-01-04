//! Rhai condition expression to Z3 constraint encoder.
//!
//! This module parses Rhai condition scripts and encodes them as Z3 formulas,
//! enabling satisfiability and validity checking for choice conditions.

use engine_script::rhai::{Engine, Expr, FnCallExpr};
use smol_str::SmolStr;
use z3::ast::{Ast, Bool, Int};
use z3::Context;

use crate::symbolic::SymbolicState;

/// Result of encoding a condition expression.
#[derive(Debug)]
pub enum EncodedCondition<'ctx> {
    /// A boolean Z3 constraint
    Bool(Bool<'ctx>),
    /// An integer Z3 expression (intermediate value for comparisons)
    Int(Int<'ctx>),
    /// The condition contains RNG and cannot be encoded deterministically
    NonDeterministic,
    /// The condition could not be parsed/encoded
    Unknown(String),
}

impl<'ctx> EncodedCondition<'ctx> {
    /// Try to get as a boolean constraint.
    pub fn as_bool(&self) -> Option<&Bool<'ctx>> {
        match self {
            EncodedCondition::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// Check if this is a non-deterministic condition.
    pub fn is_non_deterministic(&self) -> bool {
        matches!(self, EncodedCondition::NonDeterministic)
    }

    /// Check if encoding failed.
    pub fn is_unknown(&self) -> bool {
        matches!(self, EncodedCondition::Unknown(_))
    }
}

/// Encoder for Rhai conditions to Z3 constraints.
///
/// Supports common patterns:
/// - `resource("name") >= value` / `<` / `<=` / `>` / `==` / `!=`
/// - `flag("name")` (boolean check)
/// - `!flag("name")` (negated flag)
/// - `expr && expr` (conjunction)
/// - `expr || expr` (disjunction)
/// - `has_tag("category", "tag")` (encoded as fresh boolean)
/// - `reputation("faction") >= value` (like resources)
pub struct ConditionEncoder<'ctx, 'state> {
    ctx: &'ctx Context,
    state: &'state mut SymbolicState<'ctx>,
}

impl<'ctx, 'state> ConditionEncoder<'ctx, 'state> {
    /// Create a new encoder with the given Z3 context and symbolic state.
    pub fn new(ctx: &'ctx Context, state: &'state mut SymbolicState<'ctx>) -> Self {
        Self { ctx, state }
    }

    /// Encode a Rhai condition script into a Z3 constraint.
    ///
    /// Returns `EncodedCondition::Bool` for successfully encoded boolean conditions,
    /// `EncodedCondition::NonDeterministic` if RNG is used, or
    /// `EncodedCondition::Unknown` if the pattern isn't recognized.
    pub fn encode_script(&mut self, source: &str) -> EncodedCondition<'ctx> {
        // Empty condition is always true
        if source.trim().is_empty() {
            return EncodedCondition::Bool(Bool::from_bool(self.ctx, true));
        }

        // Parse the script
        let engine = Engine::new();
        let ast = match engine.compile(source) {
            Ok(ast) => ast,
            Err(e) => return EncodedCondition::Unknown(format!("Parse error: {}", e)),
        };

        // Get the statements from the AST
        let stmts = ast.statements();

        if stmts.is_empty() {
            return EncodedCondition::Unknown("No statements found".to_string());
        }

        // A condition script should be a single expression statement
        // Rhai wraps the expression in an Expr statement
        self.encode_stmt(&stmts[0])
    }

    /// Encode a statement (looking for expression statements).
    fn encode_stmt(&mut self, stmt: &engine_script::rhai::Stmt) -> EncodedCondition<'ctx> {
        use engine_script::rhai::Stmt;

        match stmt {
            // Expression statement - the main case we care about
            Stmt::Expr(expr) => self.encode_expr(expr.as_ref()),

            // Function call as statement (e.g., `flag("x")` alone)
            Stmt::FnCall(call, _) => self.encode_fn_call(call.as_ref()),

            // Other statement types we can't encode as conditions
            _ => EncodedCondition::Unknown(format!("Unsupported statement type: {:?}", std::mem::discriminant(stmt))),
        }
    }

    /// Encode a Rhai expression.
    fn encode_expr(&mut self, expr: &Expr) -> EncodedCondition<'ctx> {
        match expr {
            // Function call: resource(), flag(), has_tag(), operators, etc.
            Expr::FnCall(call, _) => self.encode_fn_call(call),

            // Boolean literal
            Expr::BoolConstant(b, _) => EncodedCondition::Bool(Bool::from_bool(self.ctx, *b)),

            // Integer literal (for intermediate use)
            Expr::IntegerConstant(n, _) => {
                EncodedCondition::Int(Int::from_i64(self.ctx, *n))
            }

            // Logical AND: lhs && rhs (can have multiple operands in StaticVec)
            Expr::And(exprs, _) => {
                let mut bools = Vec::new();
                for e in exprs.iter() {
                    match self.encode_expr(e) {
                        EncodedCondition::Bool(b) => bools.push(b),
                        EncodedCondition::NonDeterministic => return EncodedCondition::NonDeterministic,
                        other => return other,
                    }
                }
                if bools.is_empty() {
                    EncodedCondition::Bool(Bool::from_bool(self.ctx, true))
                } else {
                    let refs: Vec<&Bool<'ctx>> = bools.iter().collect();
                    EncodedCondition::Bool(Bool::and(self.ctx, &refs))
                }
            }

            // Logical OR: lhs || rhs
            Expr::Or(exprs, _) => {
                let mut bools = Vec::new();
                for e in exprs.iter() {
                    match self.encode_expr(e) {
                        EncodedCondition::Bool(b) => bools.push(b),
                        EncodedCondition::NonDeterministic => return EncodedCondition::NonDeterministic,
                        other => return other,
                    }
                }
                if bools.is_empty() {
                    EncodedCondition::Bool(Bool::from_bool(self.ctx, false))
                } else {
                    let refs: Vec<&Bool<'ctx>> = bools.iter().collect();
                    EncodedCondition::Bool(Bool::or(self.ctx, &refs))
                }
            }

            // Other expressions we can't encode
            _ => EncodedCondition::Unknown(format!("Unsupported expression type: {:?}", std::mem::discriminant(expr))),
        }
    }

    /// Encode a function call expression.
    fn encode_fn_call(&mut self, call: &FnCallExpr) -> EncodedCondition<'ctx> {
        let fn_name = call.name.as_str();

        match fn_name {
            // Resource read: resource("name")
            "resource" => {
                if let Some(name) = extract_string_arg(&call.args, 0) {
                    let var = self.state.resource(&name);
                    EncodedCondition::Int(var)
                } else {
                    EncodedCondition::Unknown("resource() requires string argument".to_string())
                }
            }

            // Flag read: flag("name") - returns boolean
            "flag" | "flag_bool" => {
                if let Some(name) = extract_string_arg(&call.args, 0) {
                    let var = self.state.flag(&name);
                    EncodedCondition::Bool(var)
                } else {
                    EncodedCondition::Unknown("flag() requires string argument".to_string())
                }
            }

            // Reputation read: reputation("faction")
            "reputation" => {
                if let Some(faction) = extract_string_arg(&call.args, 0) {
                    let var = self.state.reputation(&faction);
                    EncodedCondition::Int(var)
                } else {
                    EncodedCondition::Unknown("reputation() requires string argument".to_string())
                }
            }

            // Tag check: has_tag("category", "tag") - encode as fresh boolean
            "has_tag" => {
                if let (Some(cat), Some(tag)) = (
                    extract_string_arg(&call.args, 0),
                    extract_string_arg(&call.args, 1),
                ) {
                    // Create a unique boolean variable for this tag check
                    let var_name = format!("tag_{}_{}", cat, tag);
                    let var = Bool::new_const(self.ctx, var_name);
                    EncodedCondition::Bool(var)
                } else {
                    EncodedCondition::Unknown("has_tag() requires two string arguments".to_string())
                }
            }

            // RNG functions - cannot encode deterministically
            "rng_float" | "rng_int" => EncodedCondition::NonDeterministic,

            // Arithmetic operators: +, -, *, /
            "+" => self.encode_arithmetic(call, |_ctx, a, b| a + b),
            "-" if call.args.len() == 1 => self.encode_unary_minus(call),
            "-" => self.encode_arithmetic(call, |_ctx, a, b| a - b),
            "*" => self.encode_arithmetic(call, |_ctx, a, b| a * b),
            "/" => self.encode_arithmetic(call, |_ctx, a, b| a / b),
            "%" => self.encode_arithmetic(call, |_ctx, a, b| a % b),

            // Comparison operators: >=, >, <=, <, ==, !=
            ">=" => self.encode_comparison(call, |a, b| a.ge(&b)),
            ">" => self.encode_comparison(call, |a, b| a.gt(&b)),
            "<=" => self.encode_comparison(call, |a, b| a.le(&b)),
            "<" => self.encode_comparison(call, |a, b| a.lt(&b)),
            "==" => self.encode_equality(call, false),
            "!=" => self.encode_equality(call, true),

            // Negation: !
            "!" => {
                if let Some(inner) = call.args.first() {
                    match self.encode_expr(inner) {
                        EncodedCondition::Bool(b) => EncodedCondition::Bool(b.not()),
                        EncodedCondition::NonDeterministic => EncodedCondition::NonDeterministic,
                        other => other,
                    }
                } else {
                    EncodedCondition::Unknown("Empty negation".to_string())
                }
            }

            _ => EncodedCondition::Unknown(format!("Unknown function: {}", fn_name)),
        }
    }

    /// Encode a comparison operator (>=, >, <=, <).
    fn encode_comparison<F>(&mut self, call: &FnCallExpr, op: F) -> EncodedCondition<'ctx>
    where
        F: FnOnce(Int<'ctx>, Int<'ctx>) -> Bool<'ctx>,
    {
        if call.args.len() != 2 {
            return EncodedCondition::Unknown("Comparison requires two arguments".to_string());
        }

        let lhs = self.encode_expr(&call.args[0]);
        let rhs = self.encode_expr(&call.args[1]);

        match (lhs, rhs) {
            (EncodedCondition::Int(l), EncodedCondition::Int(r)) => {
                EncodedCondition::Bool(op(l, r))
            }
            (EncodedCondition::NonDeterministic, _) | (_, EncodedCondition::NonDeterministic) => {
                EncodedCondition::NonDeterministic
            }
            _ => EncodedCondition::Unknown("Comparison requires integer operands".to_string()),
        }
    }

    /// Encode equality (==) or inequality (!=).
    fn encode_equality(&mut self, call: &FnCallExpr, negate: bool) -> EncodedCondition<'ctx> {
        if call.args.len() != 2 {
            return EncodedCondition::Unknown("Equality requires two arguments".to_string());
        }

        let lhs = self.encode_expr(&call.args[0]);
        let rhs = self.encode_expr(&call.args[1]);

        match (lhs, rhs) {
            (EncodedCondition::Int(l), EncodedCondition::Int(r)) => {
                let eq = l._eq(&r);
                EncodedCondition::Bool(if negate { eq.not() } else { eq })
            }
            (EncodedCondition::Bool(l), EncodedCondition::Bool(r)) => {
                let eq = l._eq(&r);
                EncodedCondition::Bool(if negate { eq.not() } else { eq })
            }
            (EncodedCondition::NonDeterministic, _) | (_, EncodedCondition::NonDeterministic) => {
                EncodedCondition::NonDeterministic
            }
            _ => EncodedCondition::Unknown("Equality requires matching operand types".to_string()),
        }
    }

    /// Encode an arithmetic binary operator (+, -, *, /, %).
    fn encode_arithmetic<F>(&mut self, call: &FnCallExpr, op: F) -> EncodedCondition<'ctx>
    where
        F: FnOnce(&'ctx Context, Int<'ctx>, Int<'ctx>) -> Int<'ctx>,
    {
        if call.args.len() != 2 {
            return EncodedCondition::Unknown("Arithmetic requires two arguments".to_string());
        }

        let lhs = self.encode_expr(&call.args[0]);
        let rhs = self.encode_expr(&call.args[1]);

        match (lhs, rhs) {
            (EncodedCondition::Int(l), EncodedCondition::Int(r)) => {
                EncodedCondition::Int(op(self.ctx, l, r))
            }
            (EncodedCondition::NonDeterministic, _) | (_, EncodedCondition::NonDeterministic) => {
                EncodedCondition::NonDeterministic
            }
            _ => EncodedCondition::Unknown("Arithmetic requires integer operands".to_string()),
        }
    }

    /// Encode unary minus.
    fn encode_unary_minus(&mut self, call: &FnCallExpr) -> EncodedCondition<'ctx> {
        if let Some(inner) = call.args.first() {
            match self.encode_expr(inner) {
                EncodedCondition::Int(i) => {
                    let zero = Int::from_i64(self.ctx, 0);
                    EncodedCondition::Int(zero - i)
                }
                EncodedCondition::NonDeterministic => EncodedCondition::NonDeterministic,
                other => other,
            }
        } else {
            EncodedCondition::Unknown("Empty unary minus".to_string())
        }
    }
}

/// Extract a string constant from an expression argument.
fn extract_string_arg(args: &[Expr], index: usize) -> Option<SmolStr> {
    args.get(index).and_then(|expr| {
        if let Expr::StringConstant(s, _) = expr {
            Some(SmolStr::new(s.as_str()))
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbolic::create_context;
    use z3::SatResult;

    #[test]
    fn test_encode_empty_condition() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);
        let mut encoder = ConditionEncoder::new(&ctx, &mut state);

        let result = encoder.encode_script("");
        assert!(matches!(result, EncodedCondition::Bool(_)));

        // Empty condition is always true
        if let EncodedCondition::Bool(b) = result {
            let state2 = SymbolicState::new(&ctx);
            assert!(state2.is_valid(&b));
        }
    }

    #[test]
    fn test_encode_simple_resource_check() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);
        let mut encoder = ConditionEncoder::new(&ctx, &mut state);

        let result = encoder.encode_script(r#"resource("credits") >= 100"#);
        assert!(matches!(result, EncodedCondition::Bool(_)));
    }

    #[test]
    fn test_encode_flag_check() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);
        let mut encoder = ConditionEncoder::new(&ctx, &mut state);

        let result = encoder.encode_script(r#"flag("visited")"#);
        assert!(matches!(result, EncodedCondition::Bool(_)));
    }

    #[test]
    fn test_encode_rng_returns_non_deterministic() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);
        let mut encoder = ConditionEncoder::new(&ctx, &mut state);

        let result = encoder.encode_script(r#"rng_float() < 0.5"#);
        assert!(result.is_non_deterministic());
    }

    #[test]
    fn test_satisfiability_check() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);

        // Assert credits >= 100
        state.assert_resource_ge("credits", 100);

        // Encode condition: credits >= 50
        let mut encoder = ConditionEncoder::new(&ctx, &mut state);
        let condition = encoder.encode_script(r#"resource("credits") >= 50"#);

        // This should be satisfiable (and actually valid given credits >= 100)
        if let EncodedCondition::Bool(b) = condition {
            assert_eq!(state.check_with(&b), SatResult::Sat);
            // It's actually a tautology under the constraint
            assert!(state.is_valid(&b));
        }
    }

    #[test]
    fn test_unsatisfiability_check() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);

        // Assert credits >= 100
        state.assert_resource_ge("credits", 100);

        // Encode condition: credits < 50
        let mut encoder = ConditionEncoder::new(&ctx, &mut state);
        let condition = encoder.encode_script(r#"resource("credits") < 50"#);

        // This should be unsatisfiable (credits >= 100 && credits < 50 is false)
        if let EncodedCondition::Bool(b) = condition {
            assert_eq!(state.check_with(&b), SatResult::Unsat);
        }
    }

    #[test]
    fn test_encode_compound_condition() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);
        let mut encoder = ConditionEncoder::new(&ctx, &mut state);

        let result = encoder.encode_script(r#"resource("credits") >= 100 && flag("has_permit")"#);
        assert!(matches!(result, EncodedCondition::Bool(_)));
    }

    #[test]
    fn test_encode_negated_flag() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);

        // Set flag to true
        state.assert_flag_true("visited");

        // Encode !flag("visited")
        let mut encoder = ConditionEncoder::new(&ctx, &mut state);
        let condition = encoder.encode_script(r#"!flag("visited")"#);

        // Should be unsatisfiable since visited is true
        if let EncodedCondition::Bool(b) = condition {
            assert_eq!(state.check_with(&b), SatResult::Unsat);
        }
    }

    #[test]
    fn test_encode_arithmetic_expression() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);

        // Assert credits = 100
        state.assert_resource_eq("credits", 100);

        // Encode condition: credits + 50 >= 140
        let mut encoder = ConditionEncoder::new(&ctx, &mut state);
        let condition = encoder.encode_script(r#"resource("credits") + 50 >= 140"#);

        // 100 + 50 = 150 >= 140, so this should be valid
        if let EncodedCondition::Bool(b) = condition {
            assert!(state.is_valid(&b));
        } else {
            panic!("Expected Bool, got {:?}", condition);
        }
    }

    #[test]
    fn test_encode_arithmetic_unsat() {
        let ctx = create_context();
        let mut state = SymbolicState::new(&ctx);

        // Assert credits = 100
        state.assert_resource_eq("credits", 100);

        // Encode condition: credits * 2 < 150
        let mut encoder = ConditionEncoder::new(&ctx, &mut state);
        let condition = encoder.encode_script(r#"resource("credits") * 2 < 150"#);

        // 100 * 2 = 200 < 150 is false, so this should be unsatisfiable
        if let EncodedCondition::Bool(b) = condition {
            assert_eq!(state.check_with(&b), SatResult::Unsat);
        } else {
            panic!("Expected Bool, got {:?}", condition);
        }
    }
}
