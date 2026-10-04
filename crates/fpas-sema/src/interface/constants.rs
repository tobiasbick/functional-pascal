//! Scalar constant expressions persisted in compiled-unit interfaces.
//!
//! **Documentation:** `docs/pascal/language/types/records.md` (Default field values).

use std::collections::HashMap;

use fpas_ir::{BinaryOperation as Op, Constant, UnaryOperation};
use fpas_parser::{BinaryOp, Decl, DesignatorPart, Expr, UnaryOp, Unit};
use fpas_unit::interface::{ConstantValue, SymbolKind, UnitInterface};

/// Scalar source constants available while exporting one unit.
#[derive(Default)]
pub(crate) struct ScalarConstants {
    values: HashMap<String, Constant>,
    aliases: HashMap<String, String>,
}

impl ScalarConstants {
    /// Store a checked source constant for subsequent static pattern evaluation.
    pub(crate) fn insert(&mut self, name: &str, expression: &Expr) {
        if let Some(value) = self.evaluate(expression) {
            self.values.insert(name.to_ascii_lowercase(), value);
        }
    }

    /// Share direct-import constant values and ordinary source alias resolution.
    pub(crate) fn install(
        &mut self,
        interfaces: &[UnitInterface],
        aliases: &HashMap<String, String>,
    ) {
        self.aliases.clone_from(aliases);
        for interface in interfaces {
            for symbol in &interface.symbols {
                if let SymbolKind::Constant(Some(value)) = &symbol.kind
                    && let Some(value) = from_interface(value)
                {
                    self.values
                        .insert(symbol.qualified_name.to_ascii_lowercase(), value);
                }
            }
        }
    }
    /// Collect direct imported constants and ordered local constant declarations.
    pub(super) fn collect(
        unit: &Unit,
        interfaces: &[UnitInterface],
        aliases: &HashMap<String, String>,
    ) -> Self {
        let mut constants = Self {
            values: HashMap::new(),
            aliases: aliases.clone(),
        };
        for interface in interfaces {
            for symbol in &interface.symbols {
                if let SymbolKind::Constant(Some(value)) = &symbol.kind
                    && let Some(value) = from_interface(value)
                {
                    constants
                        .values
                        .insert(symbol.qualified_name.to_ascii_lowercase(), value);
                }
            }
        }
        for declaration in &unit.declarations {
            if let Decl::Const(definition) = declaration
                && let Some(value) = constants.evaluate(&definition.value)
            {
                constants
                    .values
                    .insert(definition.name.to_ascii_lowercase(), value);
            }
        }
        constants
    }

    /// Evaluate an expression with the runtime's scalar operator semantics.
    pub(super) fn value(&self, expression: &Expr) -> Option<ConstantValue> {
        to_interface(self.evaluate(expression)?)
    }

    /// Retrieve the evaluated value of a declared constant.
    pub(super) fn named_value(&self, name: &str) -> Option<ConstantValue> {
        to_interface(self.values.get(&name.to_ascii_lowercase())?.clone())
    }

    pub(crate) fn evaluate(&self, expression: &Expr) -> Option<Constant> {
        match expression {
            Expr::Integer(value, _) => Some(Constant::Integer(*value)),
            Expr::Real(value, _) => Some(Constant::Real(*value)),
            Expr::Bool(value, _) => Some(Constant::Boolean(*value)),
            Expr::Str(value, _) => Some(Constant::String(value.clone())),
            Expr::Paren(inner, _) => self.evaluate(inner),
            Expr::Designator(designator) => {
                let mut parts = designator
                    .parts
                    .iter()
                    .map(|part| match part {
                        DesignatorPart::Ident(name, _) => Some(name.as_str()),
                        DesignatorPart::Index(..) => None,
                    })
                    .collect::<Option<Vec<_>>>()?;
                let head = parts.first()?.to_ascii_lowercase();
                if let Some(unit) = self.aliases.get(&head) {
                    parts[0] = unit;
                }
                self.values
                    .get(&parts.join(".").to_ascii_lowercase())
                    .cloned()
            }
            Expr::UnaryOp { op, operand, .. } => {
                let value = self.evaluate(operand)?;
                let operation = match (op, &value) {
                    (UnaryOp::Negate, Constant::Integer(_)) => UnaryOperation::NegateInteger,
                    (UnaryOp::Negate, Constant::Real(_)) => UnaryOperation::NegateReal,
                    (UnaryOp::Not, Constant::Boolean(_)) => UnaryOperation::NotBoolean,
                    _ => return None,
                };
                fpas_ir::constants::fold_unary(operation, &value)
            }
            Expr::BinaryOp {
                op, left, right, ..
            } => {
                let mut left = self.evaluate(left)?;
                if matches!(
                    (op, &left),
                    (BinaryOp::And, Constant::Boolean(false))
                        | (BinaryOp::Or, Constant::Boolean(true))
                ) {
                    return Some(left);
                }
                let mut right = self.evaluate(right)?;
                if matches!(
                    (&left, &right),
                    (
                        Constant::Integer(_) | Constant::Real(_),
                        Constant::Integer(_) | Constant::Real(_)
                    )
                ) && (*op == BinaryOp::RealDiv
                    || matches!(left, Constant::Real(_))
                    || matches!(right, Constant::Real(_)))
                {
                    promote_real(&mut left);
                    promote_real(&mut right);
                }
                fpas_ir::constants::fold_binary(binary_operation(*op, &left)?, &left, &right)
            }
            _ => None,
        }
    }
}

fn to_interface(value: Constant) -> Option<ConstantValue> {
    Some(match value {
        Constant::Integer(value) => ConstantValue::Integer(value),
        Constant::Real(value) => ConstantValue::Real(value.to_bits()),
        Constant::Boolean(value) => ConstantValue::Boolean(value),
        Constant::String(value) => ConstantValue::String(value),
        Constant::Unit => return None,
    })
}

fn from_interface(value: &ConstantValue) -> Option<Constant> {
    Some(match value {
        ConstantValue::Integer(value) => Constant::Integer(*value),
        ConstantValue::Real(bits) => Constant::Real(f64::from_bits(*bits)),
        ConstantValue::Boolean(value) => Constant::Boolean(*value),
        ConstantValue::String(value) => Constant::String(value.clone()),
        ConstantValue::EnumValue { .. } => return None,
    })
}

fn promote_real(value: &mut Constant) {
    if let Constant::Integer(integer) = value {
        *value = Constant::Real(*integer as f64);
    }
}

fn binary_operation(operation: BinaryOp, left: &Constant) -> Option<Op> {
    use BinaryOp as Ast;
    Some(match (operation, left) {
        (Ast::Eq, _) => Op::Equal,
        (Ast::NotEq | Ast::Xor, _) => Op::NotEqual,
        (Ast::And, _) => Op::AndBoolean,
        (Ast::Or, _) => Op::OrBoolean,
        (Ast::Add, Constant::String(_)) => Op::ConcatString,
        (Ast::Add, Constant::Integer(_)) => Op::AddInteger,
        (Ast::Sub, Constant::Integer(_)) => Op::SubtractInteger,
        (Ast::Mul, Constant::Integer(_)) => Op::MultiplyInteger,
        (Ast::IntDiv, _) => Op::DivideInteger,
        (Ast::Mod, _) => Op::RemainderInteger,
        (Ast::Add, Constant::Real(_)) => Op::AddReal,
        (Ast::Sub, Constant::Real(_)) => Op::SubtractReal,
        (Ast::Mul, Constant::Real(_)) => Op::MultiplyReal,
        (Ast::RealDiv, _) => Op::DivideReal,
        (Ast::Lt, Constant::Integer(_)) => Op::LessThanInteger,
        (Ast::Gt, Constant::Integer(_)) => Op::GreaterThanInteger,
        (Ast::LtEq, Constant::Integer(_)) => Op::LessEqualInteger,
        (Ast::GtEq, Constant::Integer(_)) => Op::GreaterEqualInteger,
        (Ast::Lt, Constant::Real(_)) => Op::LessThanReal,
        (Ast::Gt, Constant::Real(_)) => Op::GreaterThanReal,
        (Ast::LtEq, Constant::Real(_)) => Op::LessEqualReal,
        (Ast::GtEq, Constant::Real(_)) => Op::GreaterEqualReal,
        (Ast::Lt, Constant::String(_)) => Op::LessThanString,
        (Ast::Gt, Constant::String(_)) => Op::GreaterThanString,
        (Ast::LtEq, Constant::String(_)) => Op::LessEqualString,
        (Ast::GtEq, Constant::String(_)) => Op::GreaterEqualString,
        _ => return None,
    })
}
