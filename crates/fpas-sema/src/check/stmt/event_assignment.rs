//! Event assignment checking (`B.OnClick := …` / `:= nil`).
//!
//! **Documentation:** `docs/pascal/language/types/record-events.md`

use super::Checker;
use crate::types::{EventTy, RecordTy};
use fpas_lexer::Span;
use fpas_parser::{Designator, Expr};

use super::super::context::{EventWriteInfo, PropertyReadInfo};

impl Checker {
    /// Type-checks an assignment to a record event and records its setter call.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(super) fn check_event_assignment(
        &mut self,
        target: &Designator,
        value: &Expr,
        span: Span,
        event: EventTy,
        receiver_reads: Vec<PropertyReadInfo>,
    ) {
        let clear = is_event_clear(value);
        if !clear {
            let value_ty = self.check_expr(value);
            self.check_type_compat(&event.handler_ty, &value_ty, "event assignment", span);
        }

        let key = crate::designator_lookup_key(target);
        self.event_writes.insert(
            key,
            EventWriteInfo {
                setter_name: event.setter,
                receiver_part_count: target.parts.len() - 1,
                receiver_reads,
                clear,
            },
        );
    }

    /// Finds the event `member` on `record_ty` or its canonical declaration.
    pub(crate) fn find_record_event_on_type(
        &self,
        record_ty: &RecordTy,
        member: &str,
    ) -> Option<EventTy> {
        self.find_record_member_on_type(record_ty, member, |record| &record.events)
    }
}

fn is_event_clear(expr: &Expr) -> bool {
    match expr {
        Expr::Nil(_) => true,
        Expr::Paren(inner, _) => is_event_clear(inner),
        _ => false,
    }
}
