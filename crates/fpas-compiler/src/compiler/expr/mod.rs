//! Expression lowering, including calls, literals, operators, and `try`.
//!
//! **Documentation:** `docs/pascal/02-basics.md`, `docs/pascal/04-functions.md`, `docs/pascal/07-error-handling.md` (from the repository root).

mod call;
mod special;

use crate::error::CompileError;
use fpas_bytecode::{Op, Value};
use fpas_parser::{Expr, UnaryOp};
use fpas_sema::Ty;

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
                        let operand_ty = self.ty_of(operand);
                        let negate_op = match operand_ty {
                            Ty::GenericParam(..) => Op::NegateDyn,
                            Ty::Real => Op::NegateReal,
                            _ => Op::NegateInt,
                        };
                        self.emit(negate_op, (span.line, span.column));
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
                self.emit(
                    Op::MakeArray(Self::checked_u16(elems.len(), "array elements", *span)?),
                    (span.line, span.column),
                );
            }
            Expr::DictLiteral(pairs, span) => {
                for (key, value) in pairs {
                    self.compile_expr(key)?;
                    self.compile_expr(value)?;
                }
                self.emit(
                    Op::MakeDict(Self::checked_u16(pairs.len(), "dict pairs", *span)?),
                    (span.line, span.column),
                );
            }
            Expr::RecordLiteral { fields, span } => {
                // If sema annotated this literal with a named record type that has defaults,
                // emit all fields (provided + defaults). Otherwise emit the raw fields.
                let type_name_and_specs = self.take_record_literal_expansion(expr);
                if let Some((type_name, field_specs)) = type_name_and_specs {
                    let provided: std::collections::HashMap<&str, &fpas_parser::Expr> =
                        fields.iter().map(|f| (f.name.as_str(), &f.value)).collect();
                    for (field_name, default) in &field_specs {
                        self.emit_constant(
                            Value::Str(field_name.clone()),
                            (span.line, span.column),
                        );
                        if let Some(val) = provided.get(field_name.as_str()).copied() {
                            self.compile_expr(val)?;
                        } else {
                            // Sema guaranteed that a default exists when the field is absent.
                            self.compile_expr(default.as_ref().expect(
                                "compiler: missing required field with no default — sema should have caught this",
                            ))?;
                        }
                    }
                    let n = field_specs.len() as u16;
                    let type_idx = self.chunk.add_constant(Value::Str(type_name));
                    self.emit(Op::MakeRecord(type_idx, n), (span.line, span.column));
                } else {
                    for field in fields {
                        self.emit_constant(
                            Value::Str(field.name.clone()),
                            (span.line, span.column),
                        );
                        self.compile_expr(&field.value)?;
                    }
                    let type_idx = self.chunk.add_constant(Value::Str("<record>".into()));
                    self.emit(
                        Op::MakeRecord(type_idx, fields.len() as u16),
                        (span.line, span.column),
                    );
                }
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
            Expr::Go(inner, span) => {
                self.compile_go_expr(inner, *span)?;
            }
            Expr::RecordUpdate { base, fields, span } => {
                // Emit base, then (name, value) override pairs, then UpdateRecord.
                self.compile_expr(base)?;
                for field in fields {
                    self.emit_constant(Value::Str(field.name.clone()), (span.line, span.column));
                    self.compile_expr(&field.value)?;
                }
                self.emit(
                    Op::UpdateRecord(fields.len() as u16),
                    (span.line, span.column),
                );
            }
            Expr::Error(span) => {
                self.emit(Op::Unit, (span.line, span.column));
            }
        }

        Ok(())
    }

    /// If the given `RecordLiteral` expression was annotated by sema with a named record type
    /// that has registered defaults, return the type name and the ordered field-defaults list
    /// (cloned so the borrow on `self` is released before compilation continues).
    ///
    /// Returns `None` for anonymous literals or named types without any defaults.
    fn take_record_literal_expansion(
        &self,
        expr: &Expr,
    ) -> Option<(String, Vec<(String, Option<fpas_parser::Expr>)>)> {
        let ty = self.ty_of(expr);
        if let Ty::Record(record_ty) = ty {
            if let Some(specs) = self.record_defaults.get(&record_ty.name) {
                return Some((record_ty.name.clone(), specs.clone()));
            }
        }
        None
    }
}
