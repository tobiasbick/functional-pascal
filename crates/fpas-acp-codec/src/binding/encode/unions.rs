//! Enum serialization and SDK validation witnesses for future variants.

use super::{Encoder, invalid};
use crate::{
    CodecError,
    binding::wire::Wire,
    binding::{Declaration, Shape, Type, Unknown, Value, Variant},
};

impl Encoder<'_> {
    pub(super) fn union(
        &mut self,
        declaration: &Declaration,
        variants: &[Variant],
        unknown: Option<&Unknown>,
        value: &Value,
    ) -> Result<Wire, CodecError> {
        let Value::Enum {
            name,
            variant,
            fields,
        } = value
        else {
            return Err(invalid());
        };
        if name != &declaration.name {
            return Err(invalid());
        }
        if variant == "Unknown" {
            let unknown = unknown.ok_or_else(invalid)?;
            if self.mask_unknown {
                let known = self.minimum(&Type::Named(declaration.key.clone()))?;
                return self.named(&declaration.key, &known);
            }
            return match (unknown, fields.as_slice()) {
                (Unknown::String, [Value::String(tag)])
                    if !variants.iter().any(|variant| {
                        variant
                            .tag
                            .as_ref()
                            .is_some_and(|(_, value)| value.as_str() == Some(tag))
                    }) =>
                {
                    Wire::scalar(tag)
                }
                (Unknown::Tagged(property), [Value::String(tag), Value::Raw(raw)]) => {
                    let actual = raw.member(property)?.ok_or_else(invalid)?;
                    let actual: String =
                        serde_json::from_str(actual.text()).map_err(CodecError::json)?;
                    if actual != *tag
                        || variants.iter().any(|variant| {
                            variant
                                .tag
                                .as_ref()
                                .is_some_and(|(_, value)| value.as_str() == Some(tag))
                        })
                    {
                        return Err(invalid());
                    }
                    Ok(Wire::Raw(raw.clone()))
                }
                _ => Err(invalid()),
            };
        }
        let variant = variants
            .iter()
            .find(|candidate| &candidate.name == variant)
            .ok_or_else(invalid)?;
        let payload = match (&variant.payload, fields.as_slice()) {
            (Some(ty), [value]) => Some(self.ty(ty, value)?),
            (None, []) => None,
            _ => return Err(invalid()),
        };
        if let Some((property, tag)) = &variant.tag {
            if let Some(property) = property {
                let mut fields = payload.map(Wire::object).transpose()?.unwrap_or_default();
                if fields.iter().any(|(key, _)| key == property) {
                    return Err(invalid());
                }
                fields.insert(0, (property.clone(), Wire::scalar(tag)?));
                return Ok(Wire::Object(fields));
            }
            return Wire::scalar(tag);
        }
        payload.ok_or_else(invalid)
    }

    fn minimum(&mut self, ty: &Type) -> Result<Value, CodecError> {
        Ok(match ty {
            Type::Boolean => Value::Boolean(false),
            Type::Integer { minimum, maximum } => Value::Integer(0_i64.clamp(*minimum, *maximum)),
            Type::UInt64 => Value::Unsigned(0),
            Type::Real => Value::Real(0.0),
            Type::String => Value::String(String::new()),
            Type::Raw { object } => {
                Value::Raw(crate::RawJson::parse(if *object { "{}" } else { "null" })?)
            }
            Type::Array(_) => Value::Array(Vec::new()),
            Type::Dictionary(_) => Value::Dictionary(Vec::new()),
            Type::Option(_) => Value::None,
            Type::Field(_) => Value::Missing,
            Type::Named(key) => {
                let declaration = self.catalog.declarations.get(key).ok_or_else(invalid)?;
                match &declaration.shape {
                    Shape::Scalar(Type::String) => {
                        let normalize = crate::normalizers::lookup(self.catalog.version, key);
                        let mut chosen = None;
                        for candidate in ["", "/", "application/octet-stream", "file:///", "x"] {
                            let raw = crate::codec::encode(&candidate, 4096)?;
                            if normalize.is_none_or(|normalize| normalize(&raw).is_ok()) {
                                chosen = Some(candidate.to_owned());
                                break;
                            }
                        }
                        Value::String(chosen.ok_or_else(invalid)?)
                    }
                    Shape::Scalar(ty) => self.minimum(ty)?,
                    Shape::Record(fields) => Value::Record {
                        name: declaration.name.clone(),
                        fields: fields
                            .iter()
                            .map(|field| self.minimum(&field.ty))
                            .collect::<Result<_, _>>()?,
                    },
                    Shape::Enum { variants, .. } => {
                        let first = variants.first().ok_or_else(invalid)?;
                        Value::Enum {
                            name: declaration.name.clone(),
                            variant: first.name.clone(),
                            fields: first
                                .payload
                                .as_ref()
                                .map(|ty| self.minimum(ty))
                                .transpose()?
                                .into_iter()
                                .collect(),
                        }
                    }
                    Shape::Extension { response } => Value::Record {
                        name: declaration.name.clone(),
                        fields: if *response {
                            vec![Value::Raw(crate::RawJson::parse("{}")?)]
                        } else {
                            vec![
                                Value::String("extension".into()),
                                Value::Raw(crate::RawJson::parse("{}")?),
                            ]
                        },
                    },
                }
            }
        })
    }
}
