//! Expression lowering, including calls, literals, operators, and `try`.
//!
//! **Documentation:** `docs/pascal/02-basics.md`, `docs/pascal/04-functions.md`, `docs/pascal/07-error-handling.md` (from the repository root).

mod call;
mod special;

use crate::error::CompileError;
use fpas_bytecode::{Op, Value};
use fpas_parser::{Expr, UnaryOp};

use super::Compiler;

impl Compiler {
    pub(super) fn compile_expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::Integer(n, span) => {
                self.emit_constant(Value::Integer(*n), (span.line, span.column));
            }
            Expr::Real(n, span) => {
                self.emit_constant(Value::Real(*n), (span.line, span.column));
            }
            Expr::Str(s, span) => {
                let mut chars = s.chars();
                if let (Some(c), None) = (chars.next(), chars.next()) {
                    self.emit_constant(Value::Char(c), (span.line, span.column));
                } else {
                    self.emit_constant(Value::Str(s.clone()), (span.line, span.column));
                }
            }
            Expr::Bool(b, span) => {
                self.emit_constant(Value::Boolean(*b), (span.line, span.column));
            }
            Expr::Designator(d) => {
                self.compile_designator_read(d)?;
            }
            Expr::Call {
                designator,
                args,
                span,
            } => {
                let call_key = std::ptr::from_ref(expr) as usize;
                if let Some(qualified) = self.method_calls.get(&call_key).cloned() {
                    self.compile_method_call(
                        designator,
                        &qualified,
                        args,
                        (span.line, span.column).into(),
                    )?;
                } else {
                    let name = Self::resolve_designator_name(designator);
                    self.compile_call(&name, args, (span.line, span.column).into())?;
                }
            }
            Expr::UnaryOp { op, operand, span } => {
                self.compile_expr(operand)?;
                match op {
                    UnaryOp::Negate => {
                        self.emit(Op::NegateInt, (span.line, span.column));
                    }
                    UnaryOp::Not => {
                        self.emit(Op::Not, (span.line, span.column));
                    }
                }
            }
            Expr::BinaryOp {
                op,
                left,
                right,
                span,
            } => {
                self.compile_binary_op(*op, left, right, (span.line, span.column).into())?;
            }
            Expr::Paren(inner, _) => {
                self.compile_expr(inner)?;
            }
            Expr::ArrayLiteral(elems, span) => {
                for elem in elems {
                    self.compile_expr(elem)?;
                }
                self.emit(Op::MakeArray(elems.len() as u16), (span.line, span.column));
            }
            Expr::DictLiteral(pairs, span) => {
                for (key, value) in pairs {
                    self.compile_expr(key)?;
                    self.compile_expr(value)?;
                }
                self.emit(Op::MakeDict(pairs.len() as u16), (span.line, span.column));
            }
            Expr::RecordLiteral { fields, span } => {
                for field in fields {
                    self.emit_constant(Value::Str(field.name.clone()), (span.line, span.column));
                    self.compile_expr(&field.value)?;
                }
                let type_idx = self.chunk.add_constant(Value::Str("<record>".into()));
                self.emit(
                    Op::MakeRecord(type_idx, fields.len() as u16),
                    (span.line, span.column),
                );
            }
            Expr::ResultOk(inner, span) => {
                self.compile_expr(inner)?;
                self.emit(Op::MakeOk, (span.line, span.column));
            }
            Expr::ResultError(inner, span) => {
                self.compile_expr(inner)?;
                self.emit(Op::MakeErr, (span.line, span.column));
            }
            Expr::OptionSome(inner, span) => {
                self.compile_expr(inner)?;
                self.emit(Op::MakeSome, (span.line, span.column));
            }
            Expr::OptionNone(span) => {
                self.emit(Op::MakeNone, (span.line, span.column));
            }
            Expr::Try(inner, span) => {
                self.compile_try_expr(inner, (span.line, span.column))?;
            }
            Expr::Function {
                params,
                return_type: _,
                body,
                span,
            } => {
                self.compile_function_expr(params, body, (span.line, span.column))?;
            }
            Expr::Go(inner, span) => {
                self.compile_go_expr(inner, *span)?;
            }
        }

        Ok(())
    }
}
