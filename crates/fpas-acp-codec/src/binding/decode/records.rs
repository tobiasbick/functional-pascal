//! Record defaults and raw-data preservation follow the pinned SDK declarations.

use super::Decoder;
use crate::binding::{Declaration, Field, Value, WireField};
use crate::raw::RawJsonRef;
use crate::{CodecError, RawJson};
use std::collections::HashSet;

impl Decoder<'_> {
    pub(super) fn record(
        &mut self,
        declaration: &Declaration,
        fields: &[Field],
        raw: RawJsonRef<'_>,
        normalized: Option<&RawJson>,
    ) -> Result<Value, CodecError> {
        let original = raw.members()?;
        let normalized = normalized.map(|raw| raw.view().members()).transpose()?;
        let mut values = Vec::new();
        for field in fields {
            let flattened = if matches!(field.wire, WireField::Flatten) {
                let common = fields
                    .iter()
                    .filter_map(|field| {
                        if let WireField::Property(name) = &field.wire {
                            Some(name)
                        } else {
                            None
                        }
                    })
                    .collect::<HashSet<_>>();
                Some(
                    super::super::wire::Wire::Object(
                        original
                            .iter()
                            .filter(|(name, _)| !common.contains(name))
                            .map(|(name, value)| {
                                (
                                    name.clone(),
                                    super::super::wire::Wire::Raw(value.to_owned()),
                                )
                            })
                            .collect(),
                    )
                    .finish()?,
                )
            } else {
                None
            };
            let node = match &field.wire {
                WireField::Property(name) => original
                    .iter()
                    .find(|(key, _)| key == name)
                    .map(|(_, value)| *value),
                WireField::Flatten => Some(flattened.as_ref().ok_or_else(super::invalid)?.view()),
                WireField::Extra => {
                    let mut excluded = HashSet::new();
                    for field in fields {
                        match &field.wire {
                            WireField::Property(name) => {
                                excluded.insert(name.clone());
                            }
                            WireField::Flatten => {
                                crate::binding::encode::wire_properties(
                                    self.catalog,
                                    &field.ty,
                                    &mut excluded,
                                );
                            }
                            WireField::Extra => {}
                        }
                    }
                    let extra = super::super::wire::Wire::Object(
                        original
                            .iter()
                            .filter(|(name, _)| !excluded.contains(name))
                            .map(|(name, value)| {
                                (
                                    name.clone(),
                                    super::super::wire::Wire::Raw(value.to_owned()),
                                )
                            })
                            .collect(),
                    )
                    .finish()?;
                    values.push(Value::Raw(extra));
                    continue;
                }
            };
            let decoded = self.ty(&field.ty, node, field.skip_invalid_items);
            values.push(match decoded {
                Ok(value) => value,
                Err(error) if node.is_none() || field.default_on_error => {
                    let normalized_node = match &field.wire {
                        WireField::Property(name) => normalized
                            .as_ref()
                            .and_then(|object| object.iter().find(|(key, _)| key == name))
                            .map(|(_, value)| *value),
                        _ => None,
                    };
                    if let Some(raw) = normalized_node {
                        self.ty(&field.ty, Some(raw), field.skip_invalid_items)?
                    } else if let Some(default) = &field.default {
                        let raw = crate::codec::encode(default, crate::raw::MAX_RAW_BYTES)?;
                        self.ty(&field.ty, Some(raw.view()), field.skip_invalid_items)?
                    } else {
                        self.ty(&field.ty, None, false).map_err(|_| error)?
                    }
                }
                Err(error) => return Err(error),
            });
        }
        Ok(Value::Record {
            name: declaration.name.clone(),
            fields: values,
        })
    }
}
