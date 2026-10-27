//! Checked output conversion; arbitrary JSON is emitted from its original raw text.

mod records;
mod unions;

use super::{Catalog, Shape, Type, Value, invalid, wire::Wire};
use crate::{CodecError, ErrorKind, RawJson, normalizers};
use std::collections::HashSet;

pub(super) struct Encoder<'a> {
    pub catalog: &'a Catalog,
    pub mask_unknown: bool,
    pub depth: usize,
}

/// Encode a detached nominal value after validating its complete shape with the SDK.
pub(super) fn encode(catalog: &Catalog, key: &str, value: &Value) -> Result<RawJson, CodecError> {
    Encoder {
        catalog,
        mask_unknown: false,
        depth: 0,
    }
    .named(key, value)?
    .finish()
}

pub(super) fn encode_masked(
    catalog: &Catalog,
    key: &str,
    value: &Value,
) -> Result<RawJson, CodecError> {
    Encoder {
        catalog,
        mask_unknown: true,
        depth: 0,
    }
    .named(key, value)?
    .finish()
}

pub(super) fn wire_properties(catalog: &Catalog, ty: &Type, names: &mut HashSet<String>) {
    if let Type::Named(key) = ty
        && let Some(declaration) = catalog.declarations.get(key)
    {
        match &declaration.shape {
            Shape::Record(fields) => {
                for field in fields {
                    match &field.wire {
                        super::WireField::Property(name) => {
                            names.insert(name.clone());
                        }
                        super::WireField::Flatten => wire_properties(catalog, &field.ty, names),
                        super::WireField::Extra => {}
                    }
                }
            }
            Shape::Enum { variants, unknown } => {
                for variant in variants {
                    if let Some((Some(name), _)) = &variant.tag {
                        names.insert(name.clone());
                    }
                    if let Some(ty) = &variant.payload {
                        wire_properties(catalog, ty, names);
                    }
                }
                if let Some(super::Unknown::Tagged(name)) = unknown {
                    names.insert(name.clone());
                }
            }
            _ => {}
        }
    }
}

impl Encoder<'_> {
    pub fn ty(&mut self, ty: &Type, value: &Value) -> Result<Wire, CodecError> {
        if self.depth >= crate::raw::MAX_DEPTH {
            return Err(CodecError::new(ErrorKind::DepthExceeded));
        }
        self.depth += 1;
        let result = self.inner(ty, value);
        self.depth -= 1;
        result
    }

    fn inner(&mut self, ty: &Type, value: &Value) -> Result<Wire, CodecError> {
        match (ty, value) {
            (Type::Boolean, Value::Boolean(value)) => Wire::scalar(value),
            (Type::Integer { minimum, maximum }, Value::Integer(value))
                if value >= minimum && value <= maximum =>
            {
                Wire::scalar(value)
            }
            (Type::UInt64, Value::Unsigned(value)) => Wire::scalar(value),
            (Type::Real, Value::Real(value)) if value.is_finite() => Wire::scalar(value),
            (Type::Real, Value::Real(_)) => Err(CodecError::new(ErrorKind::NonFinite)),
            (Type::String, Value::String(value)) => Wire::scalar(value),
            (Type::Raw { object }, Value::Raw(raw)) if !object || raw.text().starts_with('{') => {
                Ok(Wire::Raw(raw.clone()))
            }
            (Type::Array(inner), Value::Array(values)) => values
                .iter()
                .map(|value| self.ty(inner, value))
                .collect::<Result<_, _>>()
                .map(Wire::Array),
            (Type::Dictionary(inner), Value::Dictionary(values)) => values
                .iter()
                .map(|(name, value)| self.ty(inner, value).map(|value| (name.clone(), value)))
                .collect::<Result<_, _>>()
                .map(Wire::Object),
            (Type::Option(_), Value::None) | (Type::Field(_), Value::Null) => Wire::parse("null"),
            (Type::Option(inner), Value::Some(value))
            | (Type::Field(inner), Value::Replacement(value)) => self.ty(inner, value),
            (Type::Named(key), value) => self.named(key, value),
            _ => Err(invalid()),
        }
    }

    pub fn named(&mut self, key: &str, value: &Value) -> Result<Wire, CodecError> {
        let declaration = self.catalog.declarations.get(key).ok_or_else(invalid)?;
        let wire = match &declaration.shape {
            Shape::Scalar(ty) => self.ty(ty, value)?,
            Shape::Record(fields) => self.record(declaration, fields, value)?,
            Shape::Enum { variants, unknown } => {
                self.union(declaration, variants, unknown.as_ref(), value)?
            }
            Shape::Extension { response } => {
                let Value::Record { name, fields } = value else {
                    return Err(invalid());
                };
                if name != &declaration.name {
                    return Err(invalid());
                }
                let raw = if *response {
                    if let [Value::Raw(raw)] = fields.as_slice() {
                        raw
                    } else {
                        return Err(invalid());
                    }
                } else if let [Value::String(_), Value::Raw(raw)] = fields.as_slice() {
                    raw
                } else {
                    return Err(invalid());
                };
                Wire::Raw(raw.clone())
            }
        };
        let raw = wire.finish()?;
        if !self.mask_unknown
            && let Some(normalize) = normalizers::lookup(self.catalog.version, key)
        {
            if value.contains_unknown() {
                normalize(&encode_masked(self.catalog, key, value)?)?;
            } else {
                normalize(&raw)?;
            }
        }
        Ok(Wire::Raw(raw))
    }
}
