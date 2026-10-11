//! Entry and return checks shared by direct, indirect, imported, and method calls.
//! **Documentation:** `docs/pascal/language/functions/contracts.md`.

use super::context::LoweringContext;
use super::types::{DYNAMIC, STRING, UNIT};
use crate::CompileError;
use fpas_ir::{BlockId, Constant, LocalId, Operation, Terminator, ValueId};
use fpas_lexer::Span;
use fpas_parser::{EnsuresClause, FormalParam, RequiresClause};

/// One shared exit block receives complete return values from every return path.
#[derive(Clone, Copy)]
pub(super) struct CheckedReturn {
    /// First postcondition block.
    pub block: BlockId,
    /// Storage for a function's complete return value; absent for procedures.
    pub result: Option<LocalId>,
}

impl LoweringContext {
    /// Emit unconditional entry checks in clause order before any body statement.
    pub(super) fn lower_preconditions(
        &mut self,
        routine: &str,
        parameters: &[FormalParam],
        clauses: &[RequiresClause],
    ) -> Result<(), CompileError> {
        for clause in clauses {
            let failure = self.new_block(clause.span)?;
            let condition = self.lower_expression(&clause.condition)?;
            self.continue_if(condition, failure, clause.span)?;
            let success = self.new_block(clause.span)?;
            self.jump(success)?;
            self.switch_to(failure);
            self.contract_violation(routine, parameters, &clause.text, None, clause.span)?;
            self.switch_to(success);
        }
        Ok(())
    }

    /// Build exit checks in the heading scope, before body-local names can shadow it.
    pub(super) fn prepare_postconditions(
        &mut self,
        routine: &str,
        parameters: &[FormalParam],
        clauses: &[EnsuresClause],
    ) -> Result<(), CompileError> {
        let Some(first) = clauses.first() else {
            return Ok(());
        };
        let resume = self.current_block_id();
        let block = self.new_block(first.span)?;
        let result = if self.current_result_type() == UNIT {
            None
        } else {
            Some(self.declare_hidden_local(self.current_result_type(), first.span)?)
        };
        self.switch_to(block);
        for clause in clauses {
            self.begin_scope();
            if let (Some((name, span)), Some(result)) = (&clause.result_name, result) {
                let value = self.emit_value(
                    Operation::ReadLocal(result),
                    self.current_result_type(),
                    *span,
                )?;
                let local = self.declare_local(name, self.current_result_type(), false, *span)?;
                self.write_local(local, value, *span)?;
            }
            let failure = self.new_block(clause.span)?;
            let condition = self.lower_expression(&clause.condition)?;
            self.continue_if(condition, failure, clause.span)?;
            self.end_scope();
            let success = self.new_block(clause.span)?;
            self.jump(success)?;
            self.switch_to(failure);
            let returned = result
                .map(|local| {
                    self.emit_value(
                        Operation::ReadLocal(local),
                        self.current_result_type(),
                        clause.span,
                    )
                })
                .transpose()?;
            self.contract_violation(routine, parameters, &clause.text, returned, clause.span)?;
            self.switch_to(success);
        }
        let returned = result
            .map(|local| {
                self.emit_value(
                    Operation::ReadLocal(local),
                    self.current_result_type(),
                    first.span,
                )
            })
            .transpose()?;
        self.terminate(Terminator::Return(returned))?;
        self.switch_to(resume);
        self.checked_return = Some(CheckedReturn { block, result });
        Ok(())
    }

    /// Route ordinary and `try` returns through the shared postconditions exactly once.
    pub(in crate::lowering) fn lower_return(
        &mut self,
        value: Option<ValueId>,
        span: Span,
    ) -> Result<(), CompileError> {
        if let Some(exit) = self.checked_return {
            match (exit.result, value) {
                (Some(local), Some(value)) => self.write_local(local, value, span)?,
                (None, None) => {}
                // Keep invalid missing/value returns visible to the IR verifier.
                _ => return self.terminate(Terminator::Return(value)),
            }
            self.jump(exit.block)
        } else {
            self.terminate(Terminator::Return(value))
        }
    }

    fn contract_violation(
        &mut self,
        routine: &str,
        parameters: &[FormalParam],
        clause: &str,
        returned: Option<ValueId>,
        span: Span,
    ) -> Result<(), CompileError> {
        let mut values = Vec::with_capacity(parameters.len());
        for parameter in parameters {
            let key = self.contract_string(&parameter.name, span)?;
            let value = self.read_named_local(&parameter.name, span)?;
            let value = self.value_as(value, DYNAMIC, span)?;
            values.push((key, value));
        }
        let dictionary = self.type_table.dictionary_type(STRING, DYNAMIC, span)?;
        let values = self.emit_value(Operation::MakeDictionary(values), dictionary, span)?;
        let values = self.value_as(values, DYNAMIC, span)?;
        let mut details = Vec::with_capacity(4);
        for (key, text) in [("Routine", routine), ("Clause", clause)] {
            let key = self.contract_string(key, span)?;
            let value = self.contract_string(text, span)?;
            let value = self.value_as(value, DYNAMIC, span)?;
            details.push((key, value));
        }
        let key = self.contract_string("Parameters", span)?;
        details.push((key, values));
        if let Some(returned) = returned {
            let key = self.contract_string("Return", span)?;
            let value = self.value_as(returned, DYNAMIC, span)?;
            details.push((key, value));
        }
        let message = self.emit_value(Operation::MakeDictionary(details), dictionary, span)?;
        self.set_last_instruction_source(span)?;
        self.terminate(Terminator::Panic(message))
    }

    fn contract_string(&mut self, text: &str, span: Span) -> Result<ValueId, CompileError> {
        self.emit_value(
            Operation::Const(Constant::String(text.to_owned())),
            STRING,
            span,
        )
    }
}
