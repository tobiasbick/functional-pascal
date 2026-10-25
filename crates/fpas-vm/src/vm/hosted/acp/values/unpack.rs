//! Read native values against their expected nominal protocol declaration.

use super::{Converter, invalid};
use fpas_acp::{
    CodecError, ErrorKind, RawJson, UInt64,
    binding::{Shape, Type, Value as Detached},
};
use fpas_bytecode::{DebugType, DebugTypeId, Value};

impl Converter<'_> {
    pub(in crate::vm::hosted::acp) fn unpack(
        &mut self,
        ty: &Type,
        value: &Value,
        expected: Option<DebugTypeId>,
    ) -> Result<Detached, CodecError> {
        if self.depth >= fpas_acp::raw::MAX_DEPTH {
            return Err(CodecError::new(ErrorKind::DepthExceeded));
        }
        self.depth += 1;
        let result = self
            .expected(expected)
            .and_then(|expected| self.unpack_inner(ty, value, expected));
        self.depth -= 1;
        result
    }

    fn unpack_inner(
        &mut self,
        ty: &Type,
        value: &Value,
        expected: Option<DebugTypeId>,
    ) -> Result<Detached, CodecError> {
        Ok(match (ty, value) {
            (Type::Boolean, Value::Boolean(value)) => {
                self.scalar(expected, |ty| matches!(ty, DebugType::Boolean))?;
                Detached::Boolean(*value)
            }
            (Type::Integer { .. }, Value::Integer(value)) => {
                self.scalar(expected, |ty| matches!(ty, DebugType::Integer))?;
                Detached::Integer(*value)
            }
            (Type::Real, Value::Real(value)) if value.is_finite() => {
                self.scalar(expected, |ty| matches!(ty, DebugType::Real))?;
                Detached::Real(*value)
            }
            (Type::String, Value::Str(value)) => {
                self.scalar(expected, |ty| matches!(ty, DebugType::String))?;
                Detached::String(value.to_string())
            }
            (Type::UInt64, _) => Detached::Unsigned(
                UInt64::canonical(self.record_text(
                    value,
                    "Std.Acp.Types.UInt64",
                    "Digits",
                    expected,
                )?)?
                .get(),
            ),
            (Type::Raw { .. }, _) => Detached::Raw(RawJson::parse(self.record_text(
                value,
                "Std.Acp.Types.RawJson",
                "Text",
                expected,
            )?)?),
            (Type::Option(_), Value::OptionNone) => {
                self.element(expected, false)?;
                Detached::None
            }
            (Type::Option(inner), Value::OptionSome(value)) => {
                let id = self.element(expected, false)?;
                Detached::Some(Box::new(self.unpack(inner, value, id)?))
            }
            (Type::Field(inner), Value::Enum(value)) => {
                let owner = self.enumeration("Std.Acp.Types.Field", expected)?;
                let body = value.body();
                if body.layout.enumeration != owner {
                    return Err(invalid());
                }
                let index = self.variant(owner, &body.layout.variant)?;
                self.variant_fields(
                    index,
                    if body.layout.variant == "Value" {
                        &["Value"]
                    } else {
                        &[]
                    },
                )?;
                match (body.layout.variant.as_str(), body.values.as_slice()) {
                    ("Missing", []) => Detached::Missing,
                    ("NullValue", []) => Detached::Null,
                    ("Value", [value]) => {
                        let [id] = self.image.enum_variants[index].field_types.as_slice() else {
                            return Err(invalid());
                        };
                        Detached::Replacement(Box::new(self.unpack(inner, value, Some(*id))?))
                    }
                    _ => return Err(invalid()),
                }
            }
            (Type::Array(inner), Value::Array(values)) => {
                let id = self.element(expected, true)?;
                Detached::Array(
                    values
                        .iter()
                        .map(|value| self.unpack(inner, value, id))
                        .collect::<Result<_, _>>()?,
                )
            }
            (Type::Dictionary(inner), Value::Dict(values)) => {
                let id = self.dictionary(expected)?;
                Detached::Dictionary(
                    values
                        .iter()
                        .map(|(name, value)| {
                            let Value::Str(name) = name else {
                                return Err(invalid());
                            };
                            self.unpack(inner, value, id)
                                .map(|value| (name.to_string(), value))
                        })
                        .collect::<Result<_, _>>()?,
                )
            }
            (Type::Named(key), value) => {
                let catalog = self.catalog;
                let declaration = catalog.declarations.get(key).ok_or_else(invalid)?;
                match &declaration.shape {
                    Shape::Scalar(ty) => self.unpack(ty, value, expected)?,
                    Shape::Record(fields) => {
                        let fields = fields
                            .iter()
                            .map(|field| (field.name.clone(), field.ty.clone()))
                            .collect::<Vec<_>>();
                        self.unpack_record(&declaration.name, &fields, value, expected)?
                    }
                    Shape::Extension { response } => {
                        let fields = if *response {
                            vec![("ResultValue".into(), Type::Raw { object: false })]
                        } else {
                            vec![
                                ("Method".into(), Type::String),
                                ("Params".into(), Type::Raw { object: false }),
                            ]
                        };
                        self.unpack_record(&declaration.name, &fields, value, expected)?
                    }
                    Shape::Enum { .. } => {
                        let Value::Enum(value) = value else {
                            return Err(invalid());
                        };
                        let owner = self.enumeration(&declaration.name, expected)?;
                        let body = value.body();
                        if body.layout.enumeration != owner {
                            return Err(invalid());
                        }
                        let index = self.variant(owner, &body.layout.variant)?;
                        self.protocol_variant_fields(key, &body.layout.variant, index)?;
                        let types = self.enum_types(key, &body.layout.variant)?;
                        let metadata = self.image.enum_variants[index].field_types.clone();
                        if body.values.len() != types.len() || metadata.len() != types.len() {
                            return Err(invalid());
                        }
                        let fields = types
                            .iter()
                            .zip(&body.values)
                            .zip(metadata)
                            .map(|((ty, value), id)| self.unpack(ty, value, Some(id)))
                            .collect::<Result<_, _>>()?;
                        Detached::Enum {
                            name: declaration.name.clone(),
                            variant: body.layout.variant.clone(),
                            fields,
                        }
                    }
                }
            }
            _ => return Err(invalid()),
        })
    }

    fn unpack_record(
        &mut self,
        name: &str,
        fields: &[(String, Type)],
        value: &Value,
        expected: Option<DebugTypeId>,
    ) -> Result<Detached, CodecError> {
        let Value::Record(record) = value else {
            return Err(invalid());
        };
        let id = self.record(name, expected)?;
        let body = record.body();
        if body.layout.record != id || !body.layout.type_name.eq_ignore_ascii_case(name) {
            return Err(invalid());
        }
        let metadata = self.image.records[usize::from(id.get())].fields.clone();
        if metadata.len() != fields.len()
            || body.values.len() != fields.len()
            || body.layout.fields.len() != fields.len()
        {
            return Err(invalid());
        }
        let mut values = Vec::new();
        for (((name, ty), value), (index, field)) in fields
            .iter()
            .zip(&body.values)
            .zip(metadata.into_iter().enumerate())
        {
            if self.image.strings.get(field.name) != Some(name)
                || body.layout.fields[index] != *name
            {
                return Err(invalid());
            }
            values.push(self.unpack(ty, value, Some(field.ty))?);
        }
        Ok(Detached::Record {
            name: name.into(),
            fields: values,
        })
    }

    pub(in crate::vm::hosted::acp) fn record_text<'a>(
        &self,
        value: &'a Value,
        name: &str,
        field: &str,
        expected: Option<DebugTypeId>,
    ) -> Result<&'a str, CodecError> {
        let Value::Record(record) = value else {
            return Err(invalid());
        };
        let id = self.record(name, expected)?;
        let body = record.body();
        if body.layout.record != id
            || !body.layout.type_name.eq_ignore_ascii_case(name)
            || body.layout.fields.as_slice() != [field]
        {
            return Err(invalid());
        }
        let [Value::Str(text)] = body.values.as_slice() else {
            return Err(invalid());
        };
        Ok(text)
    }
}
