//! Delphi constructor calls `T.Create(…)` on FPAS record types.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/types/records.md`

use super::super::Checker;
use super::builtins::argument_text;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::{Applicability, Fix, ForeignHabit, SourceSpan, fix_title};
use fpas_lexer::Span;
use fpas_parser::Expr;

impl Checker {
    /// Returns the help for an unresolved `T.Create(…)` when `T` is a record type without a
    /// `Create` member, naming its fields with the written arguments.
    pub(crate) fn record_constructor_hint(&self, name: &str, args: &[Expr]) -> Option<String> {
        let (replacement, _) = self.record_constructor_form(name, args)?;
        Some(
            ForeignHabit {
                found: name,
                replacement: &replacement,
                explanation: "Records have no constructors; construct them with the type name and named fields.",
            }
            .help(),
        )
    }

    /// Returns the fix that replaces `T.Create(…)` over `span` with the record constructor,
    /// when there is one plain argument per field.
    pub(crate) fn record_constructor_fix(
        &self,
        name: &str,
        args: &[Expr],
        span: Span,
    ) -> Option<Fix> {
        let (replacement, complete) = self.record_constructor_form(name, args)?;
        if !complete {
            return None;
        }
        Fix::replace(
            fix_title(&replacement),
            Applicability::MachineApplicable,
            SourceSpan::try_from(span).ok()?,
            replacement,
        )
    }

    /// Returns the record constructor for `T.Create(…)` and whether every field got a
    /// written argument.
    fn record_constructor_form(&self, name: &str, args: &[Expr]) -> Option<(String, bool)> {
        let (type_name, member) = name.rsplit_once('.')?;
        if !member.eq_ignore_ascii_case("create") {
            return None;
        }
        let symbol = self.scopes.lookup(type_name)?;
        if symbol.kind != SymbolKind::Type {
            return None;
        }
        let Ty::Record(record) = self.resolve_visible_type(&symbol.ty) else {
            return None;
        };
        let mut complete = args.len() == record.fields.len();
        let fields = record
            .fields
            .iter()
            .enumerate()
            .map(|(index, (field, _))| {
                let value = args.get(index).and_then(argument_text).unwrap_or_else(|| {
                    complete = false;
                    "…".to_owned()
                });
                format!("{field} := {value}")
            })
            .collect::<Vec<_>>()
            .join(", ");
        Some((format!("{type_name}({fields})"), complete))
    }
}
