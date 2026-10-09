//! Declaration-bound static record values and lexical field projection.
//! See `docs/pascal/language/basics/constants.md`.

use std::collections::BTreeMap;
use std::sync::Arc;

use fpas_parser::{Designator, DesignatorPart, Expr};
use fpas_unit::interface::{RecordConstant, RecordConstantField};

use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{RecordTy, Ty};

impl Checker {
    /// Retains known scalar and record fields without materializing runtime aggregates.
    pub(crate) fn record_constant_value(&self, expression: &Expr) -> Option<Arc<RecordConstant>> {
        match expression {
            Expr::Paren(inner, _) => self.record_constant_value(inner),
            Expr::Designator(designator) => match self.constant_designator_field(designator)? {
                RecordConstantField::Record(record) => Some(record),
                RecordConstantField::Scalar(_) => None,
            },
            Expr::Call { args, .. } => {
                let record = self.constant_record_type(expression)?;
                let defaults = self.record_defaults.get(&record.name);
                let mut fields = BTreeMap::new();
                for (name, _) in &record.fields {
                    let explicit = args.iter().find(|argument| {
                        argument
                            .argument_name()
                            .is_some_and(|provided| provided.eq_ignore_ascii_case(name))
                    });
                    let value = if let Some(argument) = explicit {
                        self.constant_field_value(argument.argument_value())
                    } else {
                        let default = defaults
                            .and_then(|defaults| {
                                defaults
                                    .iter()
                                    .find(|(field, _)| field.eq_ignore_ascii_case(name))
                            })
                            .and_then(|(_, expression)| expression.as_deref());
                        default.and_then(|expression| {
                            self.record_default_values
                                .get(&Self::expr_lookup_key(expression))
                                .cloned()
                                .unwrap_or_else(|| self.constant_field_value(expression))
                        })
                    };
                    if let Some(value) = value {
                        fields.insert(name.to_ascii_lowercase(), value);
                    }
                }
                Some(Arc::new(RecordConstant { fields }))
            }
            Expr::RecordUpdate { base, fields, .. } => {
                let mut record = self.record_constant_value(base)?;
                let values = &mut Arc::make_mut(&mut record).fields;
                for field in fields {
                    let name = field.name.to_ascii_lowercase();
                    if let Some(value) = self.constant_field_value(&field.value) {
                        values.insert(name, value);
                    } else {
                        values.remove(&name);
                    }
                }
                Some(record)
            }
            _ => None,
        }
    }

    /// Resolves a typed record call, including enclosing constants hoisted for nested routines.
    pub(crate) fn constant_record_type(&self, expression: &Expr) -> Option<Arc<RecordTy>> {
        let Expr::Call { designator, .. } = expression else {
            return None;
        };
        if self
            .record_constructions
            .contains(&Self::expr_lookup_key(expression))
            && let Some(Ty::Record(record)) =
                self.expr_types.get(&Self::expr_lookup_key(expression))
        {
            return Some(Arc::clone(record));
        }
        let symbol = self
            .scopes
            .lookup(&Self::resolve_designator_name(designator))?;
        if symbol.kind != SymbolKind::Type {
            return None;
        }
        match self.resolve_visible_type(&symbol.ty) {
            Ty::Record(record) => Some(record),
            _ => None,
        }
    }

    /// Evaluates one known field while preserving shared nested-record snapshots.
    pub(crate) fn constant_field_value(&self, expression: &Expr) -> Option<RecordConstantField> {
        self.scalar_constant_value(expression)
            .map(RecordConstantField::Scalar)
            .or_else(|| {
                self.record_constant_value(expression)
                    .map(RecordConstantField::Record)
            })
    }

    /// Resolves the longest visible binding prefix before traversing record fields.
    pub(crate) fn constant_designator_field(
        &self,
        designator: &Designator,
    ) -> Option<RecordConstantField> {
        let (symbol, prefix) = self.constant_designator_binding(designator)?;
        let info = symbol.constant.as_ref()?;
        if !info.compile_time {
            return None;
        }
        let mut value = info
            .value
            .clone()
            .map(RecordConstantField::Scalar)
            .or_else(|| {
                info.record
                    .as_ref()
                    .map(|record| RecordConstantField::Record(Arc::clone(record)))
            })?;
        for part in &designator.parts[prefix..] {
            let DesignatorPart::Ident(name, _) = part else {
                return None;
            };
            let RecordConstantField::Record(record) = value else {
                return None;
            };
            value = record.fields.get(&name.to_ascii_lowercase())?.clone();
        }
        Some(value)
    }

    /// Finds the visible binding before a field path, including qualified unit imports.
    /// See `docs/pascal/language/basics/constants.md`.
    pub(crate) fn constant_designator_binding(
        &self,
        designator: &Designator,
    ) -> Option<(&Symbol, usize)> {
        let names = designator
            .parts
            .iter()
            .map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                DesignatorPart::Index(_, _) => None,
            })
            .collect::<Option<Vec<_>>>()?;
        for prefix in (1..=names.len()).rev() {
            let Some(symbol) = self.scopes.lookup(&names[..prefix].join(".")) else {
                continue;
            };
            return Some((symbol, prefix));
        }
        None
    }
}
