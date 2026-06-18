//! Expression lowering, including calls, literals, operators, and `try`.
//!
//! **Documentation:** `docs/pascal/language/basics/README.md`, `docs/pascal/04-functions.md`, `docs/pascal/07-error-handling.md` (from the repository root).

mod call;
mod constructors;
mod literals;
mod records;
mod special;

use crate::error::CompileError;
use fpas_bytecode::Op;
use fpas_parser::{Expr, UnaryOp};
use fpas_sema::Ty;

use super::Compiler;

impl Compiler {
    /// Lower an expression into bytecode.
    pub(super) fn compile_expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::Integer(..)
            | Expr::Real(..)
            | Expr::Str(..)
            | Expr::Bool(..)
            | Expr::ArrayLiteral(..)
            | Expr::DictLiteral(..) => self.compile_literal_expr(expr)?,
            Expr::Designator(d) => {
                self.compile_designator_read(d)?;
            }
            Expr::Call {
                designator,
                args,
                span,
            } => {
                let location = Self::location_of(span);
                let call_key = fpas_sema::expr_lookup_key(expr);
                if let Some(qualified) = self.method_calls.get(&call_key).cloned() {
                    self.compile_method_call(designator, &qualified, args, location)?;
                } else {
                    let name = Self::resolve_designator_name(designator);
                    self.compile_call(&name, args, location)?;
                }
            }
            Expr::UnaryOp { op, operand, span } => {
                let location = Self::location_of(span);
                self.compile_expr(operand)?;
                match op {
                    UnaryOp::Negate => {
                        let operand_ty = self.ty_of(operand);
                        let negate_op = match operand_ty {
                            Ty::GenericParam(..) => Op::NegateDyn,
                            Ty::Real => Op::NegateReal,
                            _ => Op::NegateInt,
                        };
                        self.emit(negate_op, location);
                    }
                    UnaryOp::Not => {
                        self.emit(Op::Not, location);
                    }
                }
            }
            Expr::BinaryOp {
                op,
                left,
                right,
                span,
            } => {
                self.compile_binary_op(*op, left, right, Self::location_of(span))?;
            }
            Expr::Paren(inner, _) => {
                self.compile_expr(inner)?;
            }
            Expr::RecordLiteral { .. } | Expr::RecordUpdate { .. } => {
                self.compile_record_expr(expr)?;
            }
            Expr::ResultOk(..)
            | Expr::ResultError(..)
            | Expr::OptionSome(..)
            | Expr::OptionNone(..) => self.compile_result_option_expr(expr)?,
            Expr::Try(inner, span) => {
                self.compile_try_expr(inner, Self::location_of(span))?;
            }
            Expr::Go(inner, span) => {
                self.compile_go_expr(inner, *span)?;
            }
            Expr::Error(span) => {
                self.emit(Op::Unit, Self::location_of(span));
            }
        }

        Ok(())
    }
}
