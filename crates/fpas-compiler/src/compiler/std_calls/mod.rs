mod array;
mod console;
mod conv;
mod dict;
mod math;
mod result_option;
mod str_ops;
mod task;

use crate::error::{CompileError, compile_error};
use fpas_bytecode::{Intrinsic, Op, SourceLocation};
use fpas_diagnostics::codes::{
    COMPILE_INTRINSIC_ARITY_MISMATCH, COMPILE_INVALID_MUTABLE_ARRAY_LOWERING_TARGET,
};
use fpas_parser::{DesignatorPart, Expr};

use super::{Compiler, LocalRef};

impl Compiler {
    fn local_ref_depth_slot(local_ref: LocalRef) -> (u16, u16) {
        match local_ref {
            LocalRef::Local(slot) => (0, slot),
            LocalRef::Enclosing(depth, slot) => (depth, slot),
        }
    }

    fn simple_local_ref(&self, expr: &Expr) -> Option<LocalRef> {
        let Expr::Designator(designator) = expr else {
            return None;
        };

        if designator.parts.len() != 1 {
            return None;
        }

        match &designator.parts[0] {
            DesignatorPart::Ident(name, _) => self.resolve_local(name),
            _ => None,
        }
    }

    fn call_site_span(location: SourceLocation) -> fpas_lexer::Span {
        fpas_lexer::Span {
            offset: 0,
            length: 0,
            line: location.line,
            column: location.column,
        }
    }

    fn zero_arg_error(&self, call_name: &str, location: SourceLocation) -> CompileError {
        compile_error(
            COMPILE_INTRINSIC_ARITY_MISMATCH,
            format!("{call_name} takes no arguments"),
            "Remove all arguments from this call.",
            Self::call_site_span(location),
        )
    }

    fn exact_arg_error(
        &self,
        call_name: &str,
        expected: usize,
        got: usize,
        location: SourceLocation,
    ) -> CompileError {
        compile_error(
            COMPILE_INTRINSIC_ARITY_MISMATCH,
            format!("{call_name} expects {expected} arguments, got {got}"),
            format!(
                "Call `{call_name}` with exactly {expected} argument{}.",
                if expected == 1 { "" } else { "s" }
            ),
            Self::call_site_span(location),
        )
    }

    fn invalid_mutable_array_target_error(
        &self,
        call_name: &str,
        location: SourceLocation,
    ) -> CompileError {
        compile_error(
            COMPILE_INVALID_MUTABLE_ARRAY_LOWERING_TARGET,
            format!("{call_name} requires a simple local array variable"),
            "Pass a local variable directly, not an expression.",
            Self::call_site_span(location),
        )
    }

    fn expect_zero_args(
        &self,
        call_name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        if args.is_empty() {
            Ok(())
        } else {
            Err(self.zero_arg_error(call_name, location))
        }
    }

    fn expect_exact_args(
        &self,
        call_name: &str,
        expected: usize,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        if args.len() == expected {
            Ok(())
        } else {
            Err(self.exact_arg_error(call_name, expected, args.len(), location))
        }
    }

    fn emit_intrinsic(&mut self, intrinsic: Intrinsic, location: SourceLocation) {
        self.emit(Op::Intrinsic(u16::from(intrinsic)), location);
    }

    fn emit_intrinsic_unit(&mut self, intrinsic: Intrinsic, location: SourceLocation) {
        self.emit_intrinsic(intrinsic, location);
        self.emit(Op::Unit, location);
    }

    fn mutable_array_local_ref(
        &self,
        call_name: &str,
        target: &Expr,
        location: SourceLocation,
    ) -> Result<(u16, u16), CompileError> {
        let local_ref = self
            .simple_local_ref(target)
            .ok_or_else(|| self.invalid_mutable_array_target_error(call_name, location))?;
        Ok(Self::local_ref_depth_slot(local_ref))
    }

    /// Returns `true` if this was a standard-library call (emitted here).
    pub(super) fn compile_std_library_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        if self.compile_console_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_str_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_conv_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_math_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_array_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_dict_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_result_option_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_task_call(name, args, location)? {
            return Ok(true);
        }
        Ok(false)
    }
}
