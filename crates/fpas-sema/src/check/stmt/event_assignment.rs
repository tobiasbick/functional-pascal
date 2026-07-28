//! Event assignment checking (`B.OnClick := …` / `:= nil`).
//!
//! **Documentation:** `docs/pascal/language/types/record-events.md`

use super::Checker;
use crate::types::Ty;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

use super::super::context::EventWriteInfo;

impl Checker {
    /// Returns `true` when the assignment target is a record event (handled here).
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(crate) fn try_check_event_assignment(
        &mut self,
        target: &Designator,
        value: &Expr,
        span: Span,
    ) -> bool {
        let Some((event_name, event_span)) = target.parts.last().and_then(|part| match part {
            DesignatorPart::Ident(name, part_span) => Some((name.as_str(), *part_span)),
            _ => None,
        }) else {
            return false;
        };
        if target.parts.len() < 2 {
            return false;
        }

        let only_idents = target
            .parts
            .iter()
            .all(|part| matches!(part, DesignatorPart::Ident(_, _)));
        if only_idents {
            let full_name = Self::resolve_designator_name(target);
            if self.scopes.lookup(&full_name).is_some() {
                return false;
            }
        }

        let receiver = Designator {
            parts: target.parts[..target.parts.len() - 1].to_vec(),
            span: target.span,
        };
        let receiver_key = crate::designator_lookup_key(&receiver);
        let receiver_ty = self.check_designator_expr(&receiver);
        let receiver_reads = self
            .property_reads
            .remove(&receiver_key)
            .unwrap_or_default();
        let resolved = self.resolve_visible_type(&receiver_ty);
        let Ty::Record(record_ty) = resolved else {
            return false;
        };

        if self.reject_private_record_member(&record_ty, event_name, event_span) {
            let _ = self.check_expr(value);
            return true;
        }

        let Some(event) = self.find_record_event_on_type(&record_ty, event_name) else {
            return false;
        };

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
        true
    }

    pub(crate) fn find_record_event_on_type(
        &self,
        record_ty: &crate::types::RecordTy,
        member: &str,
    ) -> Option<crate::types::EventTy> {
        if let Some((_, event)) = record_ty
            .events
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(member))
        {
            return Some(event.clone());
        }
        if record_ty.name == "<anonymous>" {
            return None;
        }
        let symbol = self.scopes.lookup(&record_ty.name)?;
        let Ty::Record(canonical) = &symbol.ty else {
            return None;
        };
        canonical
            .events
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(member))
            .map(|(_, event)| event.clone())
    }
}

fn is_event_clear(expr: &Expr) -> bool {
    match expr {
        Expr::Nil(_) => true,
        Expr::Paren(inner, _) => is_event_clear(inner),
        _ => false,
    }
}
