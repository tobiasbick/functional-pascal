//! Caller-storage modes in semantic and source signatures.
//!
//! **Documentation:** `docs/pascal/language/functions/function-types.md`

use super::{TypeTable, synthetic_span};
use crate::CompileError;
use fpas_ir::{IrType, TypeId};

impl TypeTable {
    /// Lower a semantic value/var parameter into its runtime storage type.
    pub fn parameter_type(
        &mut self,
        parameter: &fpas_sema::ParamTy,
        line: u32,
        column: u32,
    ) -> Result<TypeId, CompileError> {
        let value = self.intern(&parameter.ty, line, column)?;
        if parameter.mutable {
            self.reference_type(value, synthetic_span(line, column))
        } else {
            Ok(value)
        }
    }

    /// Intern synchronous authority for a selected logical value type.
    pub fn reference_type(
        &mut self,
        inner: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<TypeId, CompileError> {
        self.intern_kind(IrType::Reference(inner), span)
    }

    /// Lower a source formal parameter, preserving its caller-storage mode.
    pub fn formal_type(
        &mut self,
        parameter: &fpas_parser::FormalParam,
        generics: &[fpas_parser::TypeParam],
    ) -> Result<TypeId, CompileError> {
        let value = self.type_expr_with_params(&parameter.type_expr, generics)?;
        if parameter.mutable {
            self.reference_type(value, parameter.span)
        } else {
            Ok(value)
        }
    }
}
