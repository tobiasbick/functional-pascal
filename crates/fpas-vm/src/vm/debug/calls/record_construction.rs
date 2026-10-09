//! Nominal record construction with source visibility and declaration-bound defaults.
//! See `docs/pascal/language/types/records.md` and `docs/pascal/tools/debugger.md`.

use fpas_bytecode::{RecordTypeId, SharedRecord, Value};
use std::sync::Arc;

use super::detach::error;
use super::execute::CallSandbox;
use super::resolution::resolve_named;
use crate::vm::debug::evaluation::DebugCallTarget;
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

impl CallSandbox {
    /// Resolves an exact visible record type after ordinary named callables.
    pub(super) fn record_type(
        &self,
        name: &str,
    ) -> Result<Option<RecordTypeId>, DebugSessionError> {
        match resolve_named(&self.executable, &self.layouts, name) {
            Ok(_) => return Ok(None),
            Err(error) if error.kind == DebugErrorKind::UnknownCallable => {}
            Err(error) => return Err(error),
        }
        let image = self.executable.executable();
        let mut matches = image.records.iter().enumerate().filter(|(_, layout)| {
            layout.construction.as_ref().is_some_and(|info| {
                info.aliases.iter().any(|alias| {
                    alias.source == self.source
                        && image
                            .strings
                            .get(alias.name)
                            .is_some_and(|alias| alias.eq_ignore_ascii_case(name))
                })
            })
        });
        let Some((index, _)) = matches.next() else {
            return Ok(None);
        };
        if matches.next().is_some() {
            return Err(error(
                DebugErrorKind::AmbiguousCallable,
                format!("debug record type `{name}` has multiple visible nominal targets"),
                "Use an unambiguous visible type name or unit alias.",
            ));
        }
        RecordTypeId::try_from_index(index).map(Some).map_err(|_| {
            error(
                DebugErrorKind::UnknownCallable,
                "debug record type index overflows",
                "Rebuild the executable.",
            )
        })
    }

    /// Constructs one exact nominal type, evaluating omitted defaults in declaration order.
    pub(super) fn construct_record(
        &mut self,
        record: RecordTypeId,
        name: &str,
        names: &[String],
        values: Vec<Value>,
    ) -> Result<Value, DebugSessionError> {
        let image = self.executable.executable();
        let layout = &image.records[record.get() as usize];
        let info = layout.construction.as_ref().ok_or_else(|| {
            error(
                DebugErrorKind::UnavailableValue,
                "debug record has no typed construction metadata",
                "Rebuild with the current compiler.",
            )
        })?;
        if info.requires_owner
            && !info.aliases.iter().any(|alias| {
                alias.source == self.source
                    && info
                        .owner_unit
                        .and_then(|owner| image.strings.get(owner))
                        .zip(image.strings.get(alias.unit))
                        .is_some_and(|(owner, unit)| owner.eq_ignore_ascii_case(unit))
            })
        {
            return Err(error(
                DebugErrorKind::EvaluationType,
                format!(
                    "debug record `{name}` has private fields and can be constructed only in its declaring unit"
                ),
                "Use the unit's public factory outside the declaring unit.",
            ));
        }
        if names.len() != values.len() {
            return Err(error(
                DebugErrorKind::EvaluationType,
                format!("debug record `{name}` requires named fields"),
                "Construct records with named fields, for example `Point(X := 1, Y := 2)`.",
            ));
        }
        let mut ordered = vec![None; layout.fields.len()];
        for (name, value) in names.iter().zip(values) {
            let Some(index) = layout.fields.iter().position(|field| {
                image
                    .strings
                    .get(field.name)
                    .is_some_and(|field| field.eq_ignore_ascii_case(name))
            }) else {
                return Err(error(
                    DebugErrorKind::EvaluationType,
                    format!("debug record has no stored field `{name}`"),
                    "Use each declared stored field at most once.",
                ));
            };
            if ordered[index].replace(value).is_some() {
                return Err(error(
                    DebugErrorKind::EvaluationType,
                    format!("debug record field `{name}` is supplied more than once"),
                    "Supply each stored field at most once.",
                ));
            }
        }
        let field_types = layout
            .fields
            .iter()
            .map(|field| field.ty)
            .collect::<Vec<_>>();
        let defaults = info
            .defaults
            .iter()
            .map(|name| {
                name.and_then(|name| image.strings.get(name))
                    .map(str::to_owned)
            })
            .collect::<Vec<_>>();
        for (index, value) in ordered.iter().enumerate() {
            if let Some(value) = value {
                crate::vm::debug::mutation::validate_call_value(
                    &self.executable,
                    field_types[index],
                    value,
                    self.limits.max_depth,
                )
                .map_err(|failure| {
                    error(
                        DebugErrorKind::EvaluationType,
                        failure.message,
                        failure.hint,
                    )
                })?;
            } else if defaults[index].is_none() {
                let field = image
                    .strings
                    .get(layout.fields[index].name)
                    .unwrap_or("<field>");
                return Err(error(
                    DebugErrorKind::CallArity,
                    format!("debug record `{name}` is missing required field `{field}`"),
                    "Supply every field without a default value.",
                ));
            }
        }
        for (index, value) in ordered.iter_mut().enumerate() {
            if value.is_none() {
                let default = defaults[index].as_ref().ok_or_else(|| {
                    error(
                        DebugErrorKind::UnavailableValue,
                        "debug record default metadata is unavailable",
                        "Rebuild with the current compiler.",
                    )
                })?;
                *value = Some(self.invoke(DebugCallTarget::Named(default.clone()), Vec::new())?);
            }
            let field = value.as_ref().ok_or_else(|| {
                error(
                    DebugErrorKind::UnavailableValue,
                    "debug record field value is unavailable",
                    "Supply the required field.",
                )
            })?;
            crate::vm::debug::mutation::validate_call_value(
                &self.executable,
                field_types[index],
                field,
                self.limits.max_depth,
            )
            .map_err(|failure| {
                error(
                    DebugErrorKind::EvaluationType,
                    failure.message,
                    failure.hint,
                )
            })?;
        }
        let values = ordered
            .into_iter()
            .map(|value| {
                value.ok_or_else(|| {
                    error(
                        DebugErrorKind::UnavailableValue,
                        "debug record field is unavailable",
                        "Rebuild with complete construction metadata.",
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let values = self.detach_values(&values)?;
        Ok(Value::Record(SharedRecord::new(
            Arc::clone(&self.layouts.records[record.get() as usize]),
            values,
        )))
    }
}
