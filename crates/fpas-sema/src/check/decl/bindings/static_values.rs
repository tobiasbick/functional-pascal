//! Scalar static evaluation uses the same lexical declaration as expression typing.

use crate::check::Checker;
use crate::interface::{StaticEvaluationError, StaticRecord, to_scalar};
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_bytecode::{EnumTypeId, EnumVariantId, RuntimeEnumLayout, SharedEnum, Value};
use fpas_ir::Constant;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    /// Evaluate a static scalar expression without leaking shadowed local constants.
    pub(crate) fn evaluate_static_expression(&self, expression: &Expr) -> Option<Constant> {
        self.try_evaluate_static_value(expression)
            .ok()
            .flatten()
            .as_ref()
            .and_then(to_scalar)
    }

    /// Preserve static data and reached failures for enclosing comparisons and diagnostics.
    pub(in crate::check) fn try_evaluate_static_value(
        &self,
        expression: &Expr,
    ) -> Result<Option<Value>, StaticEvaluationError> {
        self.static_constants.evaluate_with(
            expression,
            &|designator| self.static_designator(designator),
            &|expression| self.static_record(expression),
        )
    }

    fn static_designator(&self, designator: &Designator) -> Option<Value> {
        let parts = designator
            .parts
            .iter()
            .map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                DesignatorPart::Index(..) => None,
            })
            .collect::<Option<Vec<_>>>()?;
        for length in (1..=parts.len()).rev() {
            let name = self.qualified_import_name(&parts[..length].join("."));
            let Some((_, symbol, declaration)) =
                self.scopes.lookup_with_scope_and_declaration(&name)
            else {
                continue;
            };
            if !matches!(symbol.kind, SymbolKind::Const | SymbolKind::EnumMember) {
                return None;
            }
            if symbol.kind == SymbolKind::EnumMember
                && let Ty::Enum(enumeration) = &symbol.ty
            {
                let variant = enumeration
                    .variants
                    .iter()
                    .find(|variant| variant.name.eq_ignore_ascii_case(parts[length - 1]))?;
                return Some(if let Some(backing) = variant.backing_value {
                    Value::Integer(backing)
                } else {
                    Value::Enum(SharedEnum::new(
                        std::sync::Arc::new(RuntimeEnumLayout {
                            enumeration: EnumTypeId::new(0),
                            variant_id: EnumVariantId::new(0),
                            type_name: String::new(),
                            variant: variant.name.to_ascii_lowercase(),
                            fields: Vec::new(),
                        }),
                        Vec::new(),
                    ))
                });
            }
            let mut value = self.static_constants.binding_value(&name, declaration)?;
            for field in &parts[length..] {
                let Value::Record(record) = value else {
                    return None;
                };
                let index = record
                    .body()
                    .layout
                    .fields
                    .iter()
                    .position(|name| name.eq_ignore_ascii_case(field))?;
                value = record.body().values.get(index)?.clone();
            }
            return Some(value);
        }
        None
    }

    fn static_record(&self, expression: &Expr) -> Option<StaticRecord> {
        let Ty::Record(record) = self.expr_types.get(&Self::expr_lookup_key(expression))? else {
            return None;
        };
        let supplied = match expression {
            Expr::RecordConstruction { fields, .. } => fields.as_slice(),
            Expr::Call { .. }
                if self
                    .record_constructions
                    .contains(&Self::expr_lookup_key(expression)) =>
            {
                &[]
            }
            _ => return None,
        };
        let defaults = self
            .record_defaults
            .get(&record.name)
            .into_iter()
            .flatten()
            .filter(|(name, _)| {
                !supplied
                    .iter()
                    .any(|field| field.name.eq_ignore_ascii_case(name))
            })
            .filter_map(|(name, value)| {
                value
                    .as_ref()?
                    .expression()
                    .map(|value| (name.clone(), value.clone()))
            })
            .collect();
        Some(StaticRecord {
            fields: record.fields.iter().map(|(name, _)| name.clone()).collect(),
            defaults,
        })
    }
}
