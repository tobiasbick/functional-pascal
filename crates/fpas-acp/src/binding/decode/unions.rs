//! Enum selection follows SDK declaration order while retaining future tags as data.

use super::{Decoder, Object, invalid};
use crate::binding::{Declaration, Unknown, Value, Variant};
use crate::{CodecError, RawJson};
use serde_json::value::RawValue;

impl Decoder<'_> {
    pub(super) fn union(
        &mut self,
        declaration: &Declaration,
        variants: &[Variant],
        unknown: Option<&Unknown>,
        raw: &RawValue,
    ) -> Result<Value, CodecError> {
        let object = serde_json::from_str::<Object<'_>>(raw.get()).ok();
        let scalar =
            serde_json::from_str::<serde_json::Value>(raw.get()).map_err(CodecError::json)?;
        if let Some(unknown) = unknown {
            let tag = match unknown {
                Unknown::String => scalar.as_str().map(str::to_owned),
                Unknown::Tagged(name) => object
                    .as_ref()
                    .and_then(|object| object.0.iter().find(|(key, _)| key == name))
                    .and_then(|(_, raw)| serde_json::from_str::<String>(raw.get()).ok()),
            };
            if let Some(tag) = tag {
                let known = variants.iter().any(|variant| {
                    variant
                        .tag
                        .as_ref()
                        .is_some_and(|(_, value)| value.as_str() == Some(&tag))
                });
                if !known {
                    return Ok(Value::Enum {
                        name: declaration.name.clone(),
                        variant: "Unknown".into(),
                        fields: match unknown {
                            Unknown::String => vec![Value::String(tag)],
                            Unknown::Tagged(_) => {
                                vec![Value::String(tag), Value::Raw(RawJson::parse(raw.get())?)]
                            }
                        },
                    });
                }
            }
        }
        for variant in variants {
            if let Some((property, expected)) = &variant.tag {
                let actual = if let Some(property) = property {
                    object
                        .as_ref()
                        .and_then(|object| object.0.iter().find(|(key, _)| key == property))
                        .and_then(|(_, raw)| serde_json::from_str(raw.get()).ok())
                        .unwrap_or_else(|| {
                            if variant.default_tag {
                                expected.clone()
                            } else {
                                serde_json::Value::Null
                            }
                        })
                } else {
                    scalar.clone()
                };
                if actual != *expected {
                    continue;
                }
            }
            let payload = match &variant.payload {
                Some(ty) => match self.ty(ty, Some(raw), false) {
                    Ok(value) => vec![value],
                    Err(_) => continue,
                },
                None => Vec::new(),
            };
            return Ok(Value::Enum {
                name: declaration.name.clone(),
                variant: variant.name.clone(),
                fields: payload,
            });
        }
        Err(invalid())
    }
}
