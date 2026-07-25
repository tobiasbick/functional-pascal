//! Lowering for `Assigned(event)` and owner-only event raises.
//!
//! **Documentation:** `docs/pascal/language/types/record-events.md`

use crate::error::{CompileError, internal_compiler_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_parser::{Designator, Expr};
use fpas_sema::{EventAssignedInfo, EventRaiseInfo};

use super::super::Compiler;

impl Compiler {
    /// Compile `Assigned(Receiver.Event)`: getter then `IsOptionSome`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(in crate::compiler) fn compile_event_assigned(
        &mut self,
        args: &[Expr],
        info: &EventAssignedInfo,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let Expr::Designator(event_designator) = args.first().ok_or_else(|| {
            internal_compiler_error(
                "`Assigned` lowering expected one event designator argument.",
                "Re-run compilation and report this internal compiler error.",
                location.line,
                location.column,
            )
        })?
        else {
            return Err(internal_compiler_error(
                "`Assigned` lowering expected a designator argument.",
                "Re-run compilation and report this internal compiler error.",
                location.line,
                location.column,
            ));
        };

        event_designator
            .parts
            .get(..info.receiver_part_count)
            .ok_or_else(|| {
                internal_compiler_error(
                    "Event-Assigned receiver metadata exceeds the designator path.",
                    "Re-run compilation and report this internal compiler error.",
                    event_designator.span.line,
                    event_designator.span.column,
                )
            })?;
        self.compile_property_receiver_prefix(
            event_designator,
            info.receiver_part_count,
            &info.receiver_reads,
        )?;
        let getter = self.qualify_name(&info.getter_name).to_string();
        let name_idx = self.add_constant(Value::Str(getter.into()), location)?;
        self.emit(Op::Call(name_idx, 1), location);
        self.emit(Op::IsOptionSome, location);
        Ok(())
    }

    /// Compile `Receiver.Event(args)`: getter, unwrap, then `CallValue`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(in crate::compiler) fn compile_event_raise(
        &mut self,
        designator: &Designator,
        args: &[Expr],
        info: &EventRaiseInfo,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        designator
            .parts
            .get(..info.receiver_part_count)
            .ok_or_else(|| {
                internal_compiler_error(
                    "Event-raise receiver metadata exceeds the designator path.",
                    "Re-run compilation and report this internal compiler error.",
                    designator.span.line,
                    designator.span.column,
                )
            })?;
        self.compile_property_receiver_prefix(
            designator,
            info.receiver_part_count,
            &info.receiver_reads,
        )?;
        let getter = self.qualify_name(&info.getter_name).to_string();
        let name_idx = self.add_constant(Value::Str(getter.into()), location)?;
        self.emit(Op::Call(name_idx, 1), location);
        self.emit(Op::UnwrapSome, location);

        // Hold the handler in a temporary local so arguments can be pushed under CallValue.
        let handler_name = format!("__event_handler_{}", self.chunk.len());
        let handler_slot = self.add_local(&handler_name, location)?;
        for arg in args {
            self.compile_expr(arg)?;
        }
        self.emit(Op::GetLocal(handler_slot), location);
        self.emit(Op::CallValue(info.arity), location);
        // Stack is `[handler, result]`; collapse to `[result]` and drop the temp local.
        self.collapse_temp_local_under_tos(handler_slot, location)?;
        Ok(())
    }
}
