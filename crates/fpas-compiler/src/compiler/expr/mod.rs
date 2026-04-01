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
                self.emit_constant(Value::Integer(*n), Self::location_of(span))?;
            }
            Expr::Real(n, span) => {
                self.emit_constant(Value::Real(*n), Self::location_of(span))?;
            }
            Expr::Str(s, span) => {
                let location = Self::location_of(span);
                let mut chars = s.chars();
                if let (Some(c), None) = (chars.next(), chars.next()) {
                    self.emit_constant(Value::Char(c), location)?;
                } else {
                    self.emit_constant(Value::Str(s.clone()), location)?;
                }
            }
            Expr::Bool(b, span) => {
                self.emit_constant(Value::Boolean(*b), Self::location_of(span))?;
            }
            Expr::Designator(d) => {
                self.compile_designator_read(d)?;
            }
            Expr::Call {
                designator,
                args,
                span,
            } => {
                let location = Self::location_of(span);
                let call_key = std::ptr::from_ref(expr) as usize;
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
            Expr::ArrayLiteral(elems, span) => {
                let location = Self::location_of(span);
                for elem in elems {
                    self.compile_expr(elem)?;
                }
                self.emit(
                    Op::MakeArray(Self::checked_u16(elems.len(), "array elements", *span)?),
                    location,
                );
            }
            Expr::DictLiteral(pairs, span) => {
                let location = Self::location_of(span);
                for (key, value) in pairs {
                    self.compile_expr(key)?;
                    self.compile_expr(value)?;
                }
                self.emit(
                    Op::MakeDict(Self::checked_u16(pairs.len(), "dict pairs", *span)?),
                    location,
                );
            }
            Expr::RecordLiteral { fields, span } => {
                let location = Self::location_of(span);
                // If sema annotated this literal with a named record type that has defaults,
                // emit all fields (provided + defaults). Otherwise emit the raw fields.
                let type_name_and_specs = self.take_record_literal_expansion(expr);
                if let Some((type_name, field_specs)) = type_name_and_specs {
                    let provided: std::collections::HashMap<&str, &fpas_parser::Expr> =
                        fields.iter().map(|f| (f.name.as_str(), &f.value)).collect();
                    for (field_name, default) in &field_specs {
                        self.emit_constant(Value::Str(field_name.clone()), location)?;
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
                    let type_idx = self.add_constant(Value::Str(type_name), location)?;
                    self.emit(Op::MakeRecord(type_idx, n), location);
                } else {
                    for field in fields {
                        self.emit_constant(Value::Str(field.name.clone()), location)?;
                        self.compile_expr(&field.value)?;
                    }
                    let type_idx = self.add_constant(Value::Str("<record>".into()), location)?;
                    self.emit(Op::MakeRecord(type_idx, fields.len() as u16), location);
                }
            }
            Expr::ResultOk(inner, span) => {
                self.compile_expr(inner)?;
                self.emit(Op::MakeOk, Self::location_of(span));
            }
            Expr::ResultError(inner, span) => {
                self.compile_expr(inner)?;
                self.emit(Op::MakeErr, Self::location_of(span));
            }
            Expr::OptionSome(inner, span) => {
                self.compile_expr(inner)?;
                self.emit(Op::MakeSome, Self::location_of(span));
            }
            Expr::OptionNone(span) => {
                self.emit(Op::MakeNone, Self::location_of(span));
            }
            Expr::Try(inner, span) => {
                self.compile_try_expr(inner, Self::location_of(span))?;
            }
            Expr::Go(inner, span) => {
                self.compile_go_expr(inner, *span)?;
            }
            Expr::RecordUpdate { base, fields, span } => {
                let location = Self::location_of(span);
                // Emit base, then (name, value) override pairs, then UpdateRecord.
                self.compile_expr(base)?;
                for field in fields {
                    self.emit_constant(Value::Str(field.name.clone()), location)?;
                    self.compile_expr(&field.value)?;
                }
                self.emit(Op::UpdateRecord(fields.len() as u16), location);
            }
            Expr::Error(span) => {
                self.emit(Op::Unit, Self::location_of(span));
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
