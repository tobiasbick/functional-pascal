//! Event access: `Assigned`, owner-only raise, and bare-read rejection.
//!
//! **Documentation:** `docs/pascal/language/types/record-events.md`

use super::super::Checker;
use crate::types::{EventTy, RecordTy, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

use super::super::context::{EventAssignedInfo, EventRaiseInfo, PropertyReadInfo};

/// Inputs required to validate and record an event raise.
pub(crate) struct EventRaiseRequest<'a> {
    /// Semantic lookup key of the call expression.
    pub(crate) call_key: usize,
    /// Full event designator, including its receiver path.
    pub(crate) designator: &'a Designator,
    /// Resolved receiver record type.
    pub(crate) record_ty: &'a RecordTy,
    /// Name of the event member being raised.
    pub(crate) event_name: &'a str,
    /// Property reads required to evaluate the receiver exactly once.
    pub(crate) receiver_reads: Vec<PropertyReadInfo>,
    /// Arguments passed to the event handler.
    pub(crate) args: &'a [Expr],
    /// Source span of the call.
    pub(crate) span: Span,
    /// Whether the raise is used as a statement.
    pub(crate) as_statement: bool,
}

impl Checker {
    /// Resolve a bare event member as an illegal value read.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(crate) fn reject_bare_event_read(
        &mut self,
        record_name: &str,
        member: &str,
        span: Span,
    ) -> Ty {
        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("Cannot read event `{record_name}.{member}` as a value"),
            format!(
                "Use `Assigned({record_name_hint}.{member})`, assign a handler, clear with `nil`, or raise it from the owning unit.",
                record_name_hint = short_type_hint(record_name)
            ),
            span,
        );
        Ty::Error
    }

    /// Type-check `Assigned(event)` when the call names the language builtin.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(crate) fn try_check_assigned_call(
        &mut self,
        call_expr: &Expr,
        designator: &Designator,
        args: &[Expr],
        span: Span,
    ) -> Option<Ty> {
        let name = Self::resolve_designator_name(designator);
        if !name.eq_ignore_ascii_case("Assigned") {
            return None;
        }
        // Prefer a user-declared `Assigned` when present.
        if self.scopes.lookup(&name).is_some() {
            return None;
        }
        if args.len() != 1 {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("`Assigned` expects 1 argument, got {}", args.len()),
                "Write `Assigned(EventDesignator)` for a record event.",
                span,
            );
            for arg in args {
                let _ = self.check_expr(arg);
            }
            return Some(Ty::Error);
        }

        let Expr::Designator(event_designator) = &args[0] else {
            let _ = self.check_expr(&args[0]);
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "`Assigned` requires an event designator",
                "Write `Assigned(Receiver.EventName)`.",
                span,
            );
            return Some(Ty::Error);
        };

        let Some((event, receiver_reads, receiver_part_count)) =
            self.resolve_event_designator(event_designator)
        else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "`Assigned` requires an event designator",
                "Write `Assigned(Receiver.EventName)`.",
                event_designator.span,
            );
            return Some(Ty::Error);
        };

        let key = Self::expr_lookup_key(call_expr);
        self.event_assigned.insert(
            key,
            EventAssignedInfo {
                getter_name: event.getter,
                receiver_part_count,
                receiver_reads,
            },
        );
        Some(Ty::Boolean)
    }

    /// Raise an event when the receiver record and member name are already resolved.
    ///
    /// Called from method-call resolution so ordinary `Std.Console.WriteLn`-style calls are not
    /// disturbed by speculative receiver checking.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(crate) fn try_check_event_raise_on_record(
        &mut self,
        request: EventRaiseRequest<'_>,
    ) -> Option<Ty> {
        let EventRaiseRequest {
            call_key,
            designator,
            record_ty,
            event_name,
            receiver_reads,
            args,
            span,
            as_statement,
        } = request;
        let event = self.find_record_event_on_type(record_ty, event_name)?;

        if !self.caller_may_raise_event(&event) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Event `{}.{}` can only be raised from its declaring unit",
                    record_ty.name, event_name
                ),
                "Assign, clear, or test the event from other units; raise it only inside the owning unit.",
                span,
            );
            self.check_args_only(args);
            return Some(Ty::Error);
        }

        let return_ty = match &event.handler_ty {
            Ty::Function(func_ty) => {
                self.check_event_handler_args(event_name, &func_ty.params, args, span);
                (*func_ty.return_type).clone()
            }
            Ty::Procedure(proc_ty) => {
                self.check_event_handler_args(event_name, &proc_ty.params, args, span);
                if as_statement {
                    Ty::Unit
                } else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Event `{}.{}` is a procedure and does not return a value",
                            record_ty.name, event_name
                        ),
                        "Raise the event as a statement, or declare a function handler type.",
                        span,
                    );
                    Ty::Error
                }
            }
            _ => {
                self.check_args_only(args);
                Ty::Error
            }
        };

        let arity = match u8::try_from(args.len()) {
            Ok(n) => n,
            Err(_) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Event `{event_name}` has too many arguments"),
                    "Reduce the number of handler arguments.",
                    span,
                );
                self.check_args_only(args);
                return Some(Ty::Error);
            }
        };

        self.event_raises.insert(
            call_key,
            EventRaiseInfo {
                getter_name: event.getter,
                receiver_part_count: designator.parts.len() - 1,
                receiver_reads,
                arity,
            },
        );

        Some(return_ty)
    }

    /// Reject an event raise wrapped in `go` because its handler may be task-bound.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(crate) fn reject_spawned_event_raise(&mut self, call_key: usize, span: Span) {
        if self.event_raises.contains_key(&call_key) {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE,
                "Cannot raise an event across a task boundary",
                "An event may contain a task-bound handler. Raise it synchronously on the current task instead of using `go`.",
                span,
            );
        }
    }

    fn check_event_handler_args(
        &mut self,
        event_name: &str,
        params: &[crate::types::ParamTy],
        args: &[Expr],
        span: Span,
    ) {
        if params.len() != args.len() {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT,
                format!(
                    "Event `{event_name}` expects {} arguments, got {}",
                    params.len(),
                    args.len()
                ),
                "Match the handler parameter list when raising the event.",
                span,
            );
        }
        for (index, arg) in args.iter().enumerate() {
            let arg_ty = self.check_expr(arg);
            if let Some(param) = params.get(index) {
                self.check_type_compat(
                    &param.ty,
                    &arg_ty,
                    &format!("event argument {}", index + 1),
                    span,
                );
            }
        }
    }

    fn resolve_event_designator(
        &mut self,
        designator: &Designator,
    ) -> Option<(EventTy, Vec<PropertyReadInfo>, usize)> {
        if designator.parts.len() < 2 {
            return None;
        }
        let event_name = match designator.parts.last()? {
            DesignatorPart::Ident(name, _) => name.clone(),
            _ => return None,
        };
        let receiver = Designator {
            parts: designator.parts[..designator.parts.len() - 1].to_vec(),
            span: designator.span,
        };
        let receiver_key = crate::designator_lookup_key(&receiver);
        let receiver_ty = self.check_designator_expr(&receiver);
        let receiver_reads = self
            .property_reads
            .remove(&receiver_key)
            .unwrap_or_default();
        let Ty::Record(record_ty) = self.resolve_visible_type(&receiver_ty) else {
            return None;
        };
        let event = self.find_record_event_on_type(&record_ty, &event_name)?;
        Some((event, receiver_reads, designator.parts.len() - 1))
    }

    fn caller_may_raise_event(&self, event: &EventTy) -> bool {
        let Some(ctx) = &self.scopes.function_ctx else {
            return event.owner_unit.is_none();
        };
        ctx.owner_unit == event.owner_unit
    }
}

fn short_type_hint(record_name: &str) -> &str {
    record_name.rsplit('.').next().unwrap_or(record_name)
}
