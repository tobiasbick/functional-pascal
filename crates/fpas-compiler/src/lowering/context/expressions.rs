//! Checked expression types and record-default declaration scope.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use super::{CompileError, LoweringContext, Ty, TypeId, internal_compiler_error};

impl LoweringContext {
    /// Evaluate a declaration-bound default without capturing caller-local names or routines.
    pub(in crate::lowering) fn lower_record_default(
        &mut self,
        expression: &fpas_parser::Expr,
        expected: TypeId,
    ) -> Result<fpas_ir::ValueId, CompileError> {
        let bindings = std::mem::take(&mut self.bindings);
        let routine = std::mem::replace(
            &mut self.program_name,
            self.source_name.to_ascii_lowercase(),
        );
        let result = self.lower_expression_as(expression, expected);
        self.bindings = bindings;
        self.program_name = routine;
        result
    }
    /// Read checked types, including retained record-default expressions.
    pub(in crate::lowering) fn expression_type(
        &self,
        expression: &fpas_parser::Expr,
    ) -> Result<Ty, CompileError> {
        self.expr_types
            .get(&fpas_sema::expr_lookup_key(expression))
            .cloned()
            .ok_or_else(|| {
                let span = expression.span();
                internal_compiler_error(
                    format!(
                        "Expression type is missing after semantic analysis for `{expression:?}`."
                    ),
                    "This is an internal compiler error. Re-run compilation and report the source program.",
                    span.line,
                    span.column,
                )
            })
    }

    /// Use semantic constructor identity before ordinary callable result inference.
    pub(in crate::lowering) fn expression_ir_type(
        &self,
        expression: &fpas_parser::Expr,
    ) -> Result<TypeId, CompileError> {
        let span = expression.span();
        if let fpas_parser::Expr::Call { designator, .. } = expression {
            let key = fpas_sema::expr_lookup_key(expression);
            if !self.intrinsic_calls.contains_key(&key) && !self.record_constructions.contains(&key)
            {
                if let Some(result) = self.member_call_result(key) {
                    return Ok(result);
                }
                let qualified = designator
                    .parts
                    .iter()
                    .map(|part| match part {
                        fpas_parser::DesignatorPart::Ident(name, _) => Some(name.as_str()),
                        fpas_parser::DesignatorPart::Index(_, _) => None,
                    })
                    .collect::<Option<Vec<_>>>()
                    .map(|parts| parts.join("."));
                if let Some(result) = qualified
                    .as_deref()
                    .and_then(|name| self.call_result_type(name))
                {
                    return Ok(result);
                }
            }
        }
        if !self
            .expr_types
            .contains_key(&fpas_sema::expr_lookup_key(expression))
            && let fpas_parser::Expr::Designator(designator) = expression
            && let Some(ty) = self.designator_type(designator)
        {
            return Ok(ty);
        }
        self.type_table
            .id(&self.expression_type(expression)?, span.line, span.column)
    }
}
