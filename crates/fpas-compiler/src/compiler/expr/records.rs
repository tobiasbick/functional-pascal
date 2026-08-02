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
                    let field_count = Self::checked_u16(field_specs.len(), "record fields", *span)?;
                    let provided: HashMap<&str, &Expr> = fields
                        .iter()
                        .map(|field| (field.name.as_str(), &field.value))
                        .collect();
                    for (field_name, default) in &field_specs {
                        self.emit_constant(Value::Str(field_name.clone().into()), location)?;
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
                    let type_name = self.qualify_name(&type_name).to_string();
                    let type_idx = self.add_constant(Value::Str(type_name.into()), location)?;
                    self.emit(Op::MakeRecord(type_idx, field_count), location);
                } else {
                    let field_count = Self::checked_u16(fields.len(), "record fields", *span)?;
                    for field in fields {
                        self.emit_constant(Value::Str(field.name.clone().into()), location)?;
                        self.compile_expr(&field.value)?;
                    }
                    let type_idx = self.add_constant(Value::Str("<record>".into()), location)?;
                    self.emit(Op::MakeRecord(type_idx, field_count), location);
                }
            }
            Expr::RecordUpdate { base, fields, span } => {
                let location = Self::location_of(span);
                let field_count = Self::checked_u16(fields.len(), "record update fields", *span)?;
                // Emit base, then (name, value) override pairs, then UpdateRecord.
                self.compile_expr(base)?;
                for field in fields {
                    self.emit_constant(Value::Str(field.name.clone().into()), location)?;
                    self.compile_expr(&field.value)?;
                }
                self.emit(Op::UpdateRecord(field_count), location);
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

#[cfg(test)]
mod tests {
    use fpas_diagnostics::codes::COMPILE_BYTECODE_OPERAND_OVERFLOW;
    use fpas_lexer::Span;
    use fpas_parser::{Expr, FieldInit};

    use super::Compiler;

    fn span() -> Span {
        Span {
            offset: 0,
            length: 0,
            line: 1,
            column: 1,
            source_id: 0,
        }
    }

    fn fields(count: usize) -> Vec<FieldInit> {
        let span = span();
        std::iter::repeat_with(|| FieldInit {
            name: "Value".to_string(),
            value: Expr::Integer(0, span),
            span,
        })
        .take(count)
        .collect()
    }

    #[test]
    fn record_field_count_accepts_u16_max_fields() {
        let fields = fields(usize::from(u16::MAX));

        let count = Compiler::checked_u16(fields.len(), "record fields", span())
            .expect("u16::MAX fields should fit the bytecode operand");

        assert_eq!(count, u16::MAX);
    }

    #[test]
    fn record_field_count_rejects_more_than_u16_max_fields() {
        let fields = fields(usize::from(u16::MAX) + 1);

        let error = Compiler::checked_u16(fields.len(), "record fields", span())
            .expect_err("a field count wider than the bytecode operand must fail");

        assert_eq!(error.code, COMPILE_BYTECODE_OPERAND_OVERFLOW);
    }

    #[test]
    fn record_literal_rejects_more_than_u16_max_fields_before_emission() {
        let span = span();
        let expression = Expr::RecordLiteral {
            fields: fields(usize::from(u16::MAX) + 1),
            span,
        };
        let mut metadata = fpas_sema::AnalysisMetadata::default();
        metadata.expr_types.insert(
            fpas_sema::expr_lookup_key(&expression),
            fpas_sema::Ty::Integer,
        );
        let mut compiler = Compiler::new(metadata);

        let error = compiler
            .compile_record_expr(&expression)
            .expect_err("a literal count wider than the bytecode operand must fail");

        assert_eq!(error.code, COMPILE_BYTECODE_OPERAND_OVERFLOW);
    }

    #[test]
    fn default_expanded_record_rejects_more_than_u16_max_fields_before_emission() {
        let (program, parse_errors) = fpas_parser::parse(
            "program RecordDefaults;
             type Big = record Value: integer := 0; end;
             begin var Item: Big := record end end.",
        );
        assert!(parse_errors.is_empty(), "{parse_errors:#?}");
        let mut metadata = fpas_sema::analyze_with_types(&program);
        assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
        metadata.record_defaults.insert(
            "Big".to_string(),
            vec![("Value".to_string(), None); usize::from(u16::MAX) + 1],
        );
        let mut compiler = Compiler::new(metadata);

        let error = compiler
            .compile_program(&program)
            .expect_err("an expanded field count wider than the bytecode operand must fail");

        assert_eq!(error.code, COMPILE_BYTECODE_OPERAND_OVERFLOW);
    }

    #[test]
    fn record_update_rejects_more_than_u16_max_fields_before_emission() {
        let span = span();
        let expression = Expr::RecordUpdate {
            base: Box::new(Expr::Integer(0, span)),
            fields: fields(usize::from(u16::MAX) + 1),
            span,
        };
        let mut compiler = Compiler::new(fpas_sema::AnalysisMetadata::default());

        let error = compiler
            .compile_record_expr(&expression)
            .expect_err("an update count wider than the bytecode operand must fail");

        assert_eq!(error.code, COMPILE_BYTECODE_OPERAND_OVERFLOW);
    }
}
