use std::collections::HashMap;

use crate::error::{CompileError, internal_compiler_error};
use fpas_bytecode::{Op, Value};
use fpas_parser::Expr;
use fpas_sema::Ty;

use super::super::Compiler;

type RecordFieldDefault = (String, Option<Expr>);
type RecordLiteralExpansion = (String, Vec<RecordFieldDefault>);

impl Compiler {
    /// Lower record literal and record update expressions.
    pub(super) fn compile_record_expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::RecordLiteral { fields, span } => {
                let location = Self::location_of(span);
                // If sema annotated this literal with a named record type that has defaults,
                // emit all fields (provided + defaults). Otherwise emit the raw fields.
                let type_name_and_specs = self.take_record_literal_expansion(expr)?;
                if let Some((type_name, field_specs)) = type_name_and_specs {
                    let provided: HashMap<&str, &Expr> = fields
                        .iter()
                        .map(|field| (field.name.as_str(), &field.value))
                        .collect();
                    for (field_name, default) in &field_specs {
                        self.emit_constant(Value::Str(field_name.clone()), location)?;
                        if let Some(value) = provided.get(field_name.as_str()).copied() {
                            self.compile_expr(value)?;
                        } else {
                            let Some(default_expr) = default.as_ref() else {
                                return Err(internal_compiler_error(
                                    "Missing required record field with no default value (semantic analysis should have reported this).",
                                    "This is an internal compiler error. Re-run compilation and report the source program.",
                                    span.line,
                                    span.column,
                                ));
                            };
                            self.compile_expr(default_expr)?;
                        }
                    }
                    let field_count = field_specs.len() as u16;
                    let type_idx = self.add_constant(Value::Str(type_name), location)?;
                    self.emit(Op::MakeRecord(type_idx, field_count), location);
                } else {
                    for field in fields {
                        self.emit_constant(Value::Str(field.name.clone()), location)?;
                        self.compile_expr(&field.value)?;
                    }
                    let type_idx = self.add_constant(Value::Str("<record>".into()), location)?;
                    self.emit(Op::MakeRecord(type_idx, fields.len() as u16), location);
                }
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
            other => {
                let span = other.span();
                return Err(internal_compiler_error(
                    "Compiler routed a non-record expression to record lowering.",
                    "This is an internal compiler error. Re-run compilation and report the source program.",
                    span.line,
                    span.column,
                ));
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
    ) -> Result<Option<RecordLiteralExpansion>, CompileError> {
        let ty = self.ty_of(expr)?;
        let Ty::Record(record_ty) = ty else {
            return Ok(None);
        };
        if record_ty.name == "<anonymous>" {
            return Ok(None);
        }

        if let Some(specs) = self.record_defaults.get(&record_ty.name) {
            return Ok(Some((record_ty.name.clone(), specs.clone())));
        }

        let Expr::RecordLiteral { fields, .. } = expr else {
            return Ok(None);
        };
        Ok(Some((
            record_ty.name.clone(),
            fields
                .iter()
                .map(|field| (field.name.clone(), None))
                .collect(),
        )))
    }
}
