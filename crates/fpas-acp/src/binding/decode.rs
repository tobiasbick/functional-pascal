//! SDK-validated conversion from wire data to detached protocol values.

mod records;
mod unions;

use super::{Catalog, Shape, Type, Value, build::invalid, wire::Object};
use crate::{CodecError, ErrorKind, RawJson, normalizers};
use serde_json::value::RawValue;

pub(super) struct Decoder<'a> {
    pub catalog: &'a Catalog,
    pub depth: usize,
}

impl Catalog {
    /// Decode one named protocol value with the pinned SDK's defaults and validation.
    /// Arbitrary subtrees and unknown variants retain their original JSON text.
    pub fn decode(&self, key: &str, raw: &RawJson) -> Result<Value, CodecError> {
        Decoder {
            catalog: self,
            depth: 0,
        }
        .named(key, raw)
    }
}

impl Decoder<'_> {
    pub fn ty(
        &mut self,
        ty: &Type,
        raw: Option<&RawValue>,
        skip_invalid: bool,
    ) -> Result<Value, CodecError> {
        if self.depth >= crate::raw::MAX_DEPTH {
            return Err(CodecError::new(ErrorKind::DepthExceeded));
        }
        self.depth += 1;
        let result = self.inner(ty, raw, skip_invalid);
        self.depth -= 1;
        result
    }

    fn inner(
        &mut self,
        ty: &Type,
        raw: Option<&RawValue>,
        skip_invalid: bool,
    ) -> Result<Value, CodecError> {
        match ty {
            Type::Option(inner) => {
                return match raw.filter(|raw| raw.get() != "null") {
                    None => Ok(Value::None),
                    Some(raw) => self
                        .ty(inner, Some(raw), skip_invalid)
                        .map(|value| Value::Some(Box::new(value))),
                };
            }
            Type::Field(inner) => {
                return match raw {
                    None => Ok(Value::Missing),
                    Some(raw) if raw.get() == "null" => Ok(Value::Null),
                    Some(raw) => self
                        .ty(inner, Some(raw), skip_invalid)
                        .map(|value| Value::Replacement(Box::new(value))),
                };
            }
            _ => {}
        }
        let raw = raw.ok_or_else(invalid)?;
        Ok(match ty {
            Type::Boolean => {
                Value::Boolean(serde_json::from_str(raw.get()).map_err(CodecError::json)?)
            }
            Type::String => {
                Value::String(serde_json::from_str(raw.get()).map_err(CodecError::json)?)
            }
            Type::Integer { minimum, maximum } => {
                let value: i64 = serde_json::from_str(raw.get()).map_err(CodecError::json)?;
                if value < *minimum || value > *maximum {
                    return Err(CodecError::new(ErrorKind::InvalidInteger));
                }
                Value::Integer(value)
            }
            Type::UInt64 => {
                Value::Unsigned(serde_json::from_str(raw.get()).map_err(CodecError::json)?)
            }
            Type::Real => {
                let value: f64 = serde_json::from_str(raw.get()).map_err(CodecError::json)?;
                if !value.is_finite() {
                    return Err(CodecError::new(ErrorKind::NonFinite));
                }
                Value::Real(value)
            }
            Type::Raw { object } => {
                if *object && !raw.get().starts_with('{') {
                    return Err(invalid());
                }
                Value::Raw(RawJson::parse(raw.get())?)
            }
            Type::Array(inner) => {
                let items: Vec<&RawValue> =
                    serde_json::from_str(raw.get()).map_err(CodecError::json)?;
                let mut values = Vec::new();
                for item in items {
                    match self.ty(inner, Some(item), false) {
                        Ok(value) => values.push(value),
                        Err(_) if skip_invalid => {}
                        Err(error) => return Err(error),
                    }
                }
                Value::Array(values)
            }
            Type::Dictionary(inner) => {
                let object: Object<'_> =
                    serde_json::from_str(raw.get()).map_err(CodecError::json)?;
                Value::Dictionary(
                    object
                        .0
                        .into_iter()
                        .map(|(name, raw)| {
                            self.ty(inner, Some(raw), false).map(|value| (name, value))
                        })
                        .collect::<Result<_, _>>()?,
                )
            }
            Type::Named(key) => self.named(key, &RawJson::parse(raw.get())?)?,
            Type::Option(_) | Type::Field(_) => return Err(invalid()),
        })
    }

    pub fn named(&mut self, key: &str, raw: &RawJson) -> Result<Value, CodecError> {
        let declaration = self.catalog.declarations.get(key).ok_or_else(invalid)?;
        let normalized =
            normalizers::lookup(self.catalog.version, key).map(|normalize| normalize(raw));
        let node: &RawValue = serde_json::from_str(raw.text()).map_err(CodecError::json)?;
        let value = match &declaration.shape {
            Shape::Scalar(ty) => self.ty(ty, Some(node), false)?,
            Shape::Record(fields) => self.record(
                declaration,
                fields,
                raw,
                normalized.as_ref().and_then(|result| result.as_ref().ok()),
            )?,
            Shape::Enum { variants, unknown } => {
                self.union(declaration, variants, unknown.as_ref(), node)?
            }
            Shape::Extension { response } => Value::Record {
                name: declaration.name.clone(),
                fields: if *response {
                    vec![Value::Raw(raw.clone())]
                } else {
                    vec![Value::String(String::new()), Value::Raw(raw.clone())]
                },
            },
        };
        if let Some(Err(error)) = normalized {
            if !value.contains_unknown() {
                return Err(error);
            }
            let placeholder = self.catalog.encode_masked(key, &value)?;
            normalizers::lookup(self.catalog.version, key).ok_or_else(invalid)?(&placeholder)?;
        }
        Ok(value)
    }
}
