//! Record defaults and raw-data preservation follow the pinned SDK declarations.

use super::{Decoder, Object};
use crate::binding::{Declaration, Field, Value, WireField};
use crate::{CodecError, RawJson};
use serde_json::value::RawValue;
use std::collections::HashSet;

impl Decoder<'_> {
    pub(super) fn record(
        &mut self,
        declaration: &Declaration,
        fields: &[Field],
        raw: &RawJson,
        normalized: Option<&RawJson>,
    ) -> Result<Value, CodecError> {
        let original: Object<'_> = serde_json::from_str(raw.text()).map_err(CodecError::json)?;
        let normalized: Option<Object<'_>> = normalized
            .map(|raw| serde_json::from_str(raw.text()))
            .transpose()
            .map_err(CodecError::json)?;
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
                            .0
                            .iter()
                            .filter(|(name, _)| !common.contains(name))
                            .map(|(name, value)| {
                                Ok((name.clone(), super::super::wire::Wire::parse(value.get())?))
                            })
                            .collect::<Result<_, CodecError>>()?,
                    )
                    .finish()?,
                )
            } else {
                None
            };
            let node = match &field.wire {
                WireField::Property(name) => original
                    .0
                    .iter()
                    .find(|(key, _)| key == name)
                    .map(|(_, value)| *value),
                WireField::Flatten => Some(
                    serde_json::from_str::<&RawValue>(
                        flattened.as_ref().ok_or_else(super::invalid)?.text(),
                    )
                    .map_err(CodecError::json)?,
                ),
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
                            .0
                            .iter()
                            .filter(|(name, _)| !excluded.contains(name))
                            .map(|(name, value)| {
                                Ok((name.clone(), super::super::wire::Wire::parse(value.get())?))
                            })
                            .collect::<Result<_, CodecError>>()?,
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
                            .and_then(|object| object.0.iter().find(|(key, _)| key == name))
                            .map(|(_, value)| *value),
                        _ => None,
                    };
                    if let Some(raw) = normalized_node {
                        self.ty(&field.ty, Some(raw), field.skip_invalid_items)?
                    } else if let Some(default) = &field.default {
                        let raw = crate::codec::encode(default, crate::raw::MAX_RAW_BYTES)?;
                        self.ty(
                            &field.ty,
                            Some(serde_json::from_str(raw.text()).map_err(CodecError::json)?),
                            field.skip_invalid_items,
                        )?
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
