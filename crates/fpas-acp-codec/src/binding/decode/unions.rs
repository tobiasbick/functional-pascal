//! Enum selection follows SDK declaration order while retaining future tags as data.

use super::{Decoder, invalid};
use crate::CodecError;
use crate::binding::{Declaration, Unknown, Value, Variant};
use crate::raw::RawJsonRef;

impl Decoder<'_> {
    pub(super) fn union(
        &mut self,
        declaration: &Declaration,
        variants: &[Variant],
        unknown: Option<&Unknown>,
        raw: RawJsonRef<'_>,
    ) -> Result<Value, CodecError> {
        let object = raw.members().ok();
        let scalar =
            serde_json::from_str::<serde_json::Value>(raw.text()).map_err(CodecError::json)?;
        if let Some(unknown) = unknown {
            let tag = match unknown {
                Unknown::String => scalar.as_str().map(str::to_owned),
                Unknown::Tagged(name) => object
                    .as_ref()
                    .and_then(|object| object.iter().find(|(key, _)| key == name))
                    .and_then(|(_, raw)| serde_json::from_str::<String>(raw.text()).ok()),
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
                                vec![Value::String(tag), Value::Raw(raw.to_owned())]
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
                        .and_then(|object| object.iter().find(|(key, _)| key == property))
                        .and_then(|(_, raw)| serde_json::from_str(raw.text()).ok())
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
