//! Finite constructor domains shared by coverage and scalar range matching.
//! **Documentation:** `docs/pascal/language/pattern-matching/exhaustiveness.md`.

use super::{Checker, Ty};
use fpas_unit::interface::ConstantValue;

/// One finite constructor and the scalar value of a fieldless constructor, if any.
pub(super) struct Constructor {
    pub key: String,
    pub display: String,
    pub fields: Vec<Ty>,
    pub value: Option<ConstantValue>,
}

impl Checker {
    /// Resolve the checked constructor's payload types and optional scalar identity.
    pub(super) fn constructor(&self, ty: &Ty, key: &str, arity: usize) -> Constructor {
        self.constructors(ty)
            .and_then(|constructors| {
                constructors
                    .into_iter()
                    .find(|constructor| constructor.key == key && constructor.fields.len() == arity)
            })
            .unwrap_or_else(|| Constructor {
                key: key.into(),
                display: key.into(),
                fields: vec![Ty::Error; arity],
                value: None,
            })
    }

    /// Finite constructors of a resolved enum, option, result, or Boolean domain.
    pub(super) fn constructors(&self, ty: &Ty) -> Option<Vec<Constructor>> {
        let resolved = self.resolve_visible_type(ty);
        if let Some(enum_ty) = self.resolve_enum_ty(&resolved) {
            return Some(
                enum_ty
                    .variants
                    .iter()
                    .map(|variant| Constructor {
                        key: variant.name.to_ascii_lowercase(),
                        display: format!("{}.{}", enum_ty.name, variant.name),
                        fields: variant.fields.iter().map(|(_, ty)| ty.clone()).collect(),
                        value: variant.backing_value.map(|backing_value| {
                            ConstantValue::EnumValue {
                                enum_name: enum_ty.name.to_ascii_lowercase(),
                                variant_name: variant.name.to_ascii_lowercase(),
                                backing_value,
                            }
                        }),
                    })
                    .collect(),
            );
        }
        let constructor = |key: &str, display: &str, fields: Vec<Ty>, value| Constructor {
            key: key.into(),
            display: display.into(),
            fields,
            value,
        };
        match super::super::conversion_labels::comparison_type(&resolved) {
            Ty::Option(inner) => Some(vec![
                constructor("some", "Some", vec![(**inner).clone()], None),
                constructor("none", "None", Vec::new(), None),
            ]),
            Ty::Result(ok, error) => Some(vec![
                constructor("ok", "Ok", vec![(**ok).clone()], None),
                constructor("error", "Error", vec![(**error).clone()], None),
            ]),
            Ty::Boolean => Some(vec![
                constructor(
                    "true",
                    &boolean_display(&resolved, true),
                    Vec::new(),
                    Some(ConstantValue::Boolean(true)),
                ),
                constructor(
                    "false",
                    &boolean_display(&resolved, false),
                    Vec::new(),
                    Some(ConstantValue::Boolean(false)),
                ),
            ]),
            _ => None,
        }
    }
}

fn boolean_display(ty: &Ty, value: bool) -> String {
    match ty {
        Ty::Distinct(distinct) => format!("{}({value})", distinct.name),
        _ => value.to_string(),
    }
}
