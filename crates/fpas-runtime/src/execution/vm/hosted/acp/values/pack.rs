//! Materialize detached values only after their complete protocol shape was validated.

use super::{Converter, invalid};
use crate::{DebugType, DebugTypeId, SharedEnum, SharedRecord, Value};
use fpas_acp::{
    CodecError, ErrorKind,
    binding::{Shape, Type, Value as Detached},
};

impl Converter<'_> {
    pub(in crate::execution::vm::hosted::acp) fn pack(
        &mut self,
        ty: &Type,
        value: Detached,
        expected: Option<DebugTypeId>,
    ) -> Result<Value, CodecError> {
        if self.depth >= fpas_acp::raw::MAX_DEPTH {
            return Err(CodecError::new(ErrorKind::DepthExceeded));
        }
        self.depth += 1;
        let result = self
            .expected(expected)
            .and_then(|expected| self.pack_inner(ty, value, expected));
        self.depth -= 1;
        result
    }

    fn pack_inner(
        &mut self,
        ty: &Type,
        value: Detached,
        expected: Option<DebugTypeId>,
    ) -> Result<Value, CodecError> {
        Ok(match (ty, value) {
            (Type::Boolean, Detached::Boolean(value)) => {
                self.scalar(expected, |ty| matches!(ty, DebugType::Boolean))?;
                Value::Boolean(value)
            }
            (Type::Integer { .. }, Detached::Integer(value)) => {
                self.scalar(expected, |ty| matches!(ty, DebugType::Integer))?;
                Value::Integer(value)
            }
            (Type::Real, Detached::Real(value)) if value.is_finite() => {
                self.scalar(expected, |ty| matches!(ty, DebugType::Real))?;
                Value::Real(value)
            }
            (Type::String, Detached::String(value)) => {
                self.scalar(expected, |ty| matches!(ty, DebugType::String))?;
                Value::Str(value.into())
            }
            (Type::UInt64, Detached::Unsigned(value)) => self.text_record(
                "Std.Acp.Types.UInt64",
                "Digits",
                value.to_string(),
                expected,
            )?,
            (Type::Raw { .. }, Detached::Raw(value)) => self.text_record(
                "Std.Acp.Types.RawJson",
                "Text",
                value.text().to_owned(),
                expected,
            )?,
            (Type::Option(_), Detached::None) => {
                self.element(expected, false)?;
                Value::OptionNone
            }
            (Type::Option(inner), Detached::Some(value)) => {
                let id = self.element(expected, false)?;
                Value::option_some(self.pack(inner, *value, id)?)
            }
            (Type::Field(inner), value) => {
                let owner = self.enumeration("Std.Acp.Types.Field", expected)?;
                let (variant, values) = match value {
                    Detached::Missing => ("Missing", Vec::new()),
                    Detached::Null => ("NullValue", Vec::new()),
                    Detached::Replacement(value) => {
                        let index = self.variant(owner, "Value")?;
                        let types = &self.image.enum_variants[index].field_types;
                        let [id] = types.as_slice() else {
                            return Err(invalid());
                        };
                        ("Value", vec![self.pack(inner, *value, Some(*id))?])
                    }
                    _ => return Err(invalid()),
                };
                let index = self.variant(owner, variant)?;
                self.variant_fields(index, if variant == "Value" { &["Value"] } else { &[] })?;
                self.enum_value(index, values)?
            }
            (Type::Array(inner), Detached::Array(values)) => {
                let id = self.element(expected, true)?;
                Value::Array(
                    values
                        .into_iter()
                        .map(|value| self.pack(inner, value, id))
                        .collect::<Result<Vec<_>, _>>()?
                        .into(),
                )
            }
            (Type::Dictionary(inner), Detached::Dictionary(values)) => {
                let id = self.dictionary(expected)?;
                Value::dict(
                    values
                        .into_iter()
                        .map(|(name, value)| {
                            self.pack(inner, value, id)
                                .map(|value| (Value::Str(name.into()), value))
                        })
                        .collect::<Result<_, _>>()?,
                )
            }
            (Type::Named(key), value) => {
                let catalog = self.catalog;
                let declaration = catalog.declarations.get(key).ok_or_else(invalid)?;
                match &declaration.shape {
                    Shape::Scalar(ty) => self.pack(ty, value, expected)?,
                    Shape::Record(fields) => {
                        let Detached::Record {
                            name,
                            fields: values,
                        } = value
                        else {
                            return Err(invalid());
                        };
                        if name != declaration.name {
                            return Err(invalid());
                        }
                        self.pack_record(
                            &name,
                            &fields
                                .iter()
                                .map(|field| (field.name.clone(), field.ty.clone()))
                                .collect::<Vec<_>>(),
                            values,
                            expected,
                        )?
                    }
                    Shape::Extension { response } => {
                        let Detached::Record {
                            name,
                            fields: values,
                        } = value
                        else {
                            return Err(invalid());
                        };
                        if name != declaration.name {
                            return Err(invalid());
                        }
                        let fields = if *response {
                            vec![("ResultValue".into(), Type::Raw { object: false })]
                        } else {
                            vec![
                                ("Method".into(), Type::String),
                                ("Params".into(), Type::Raw { object: false }),
                            ]
                        };
                        self.pack_record(&name, &fields, values, expected)?
                    }
                    Shape::Enum { .. } => {
                        let Detached::Enum {
                            name,
                            variant,
                            fields: values,
                        } = value
                        else {
                            return Err(invalid());
                        };
                        if name != declaration.name {
                            return Err(invalid());
                        }
                        let owner = self.enumeration(&name, expected)?;
                        let index = self.variant(owner, &variant)?;
                        self.protocol_variant_fields(key, &variant, index)?;
                        let types = self.enum_types(key, &variant)?;
                        let metadata = self.image.enum_variants[index].field_types.clone();
                        if values.len() != types.len() || metadata.len() != types.len() {
                            return Err(invalid());
                        }
                        let values = types
                            .iter()
                            .zip(values)
                            .zip(metadata)
                            .map(|((ty, value), id)| self.pack(ty, value, Some(id)))
                            .collect::<Result<_, _>>()?;
                        self.enum_value(index, values)?
                    }
                }
            }
            _ => return Err(invalid()),
        })
    }

    fn pack_record(
        &mut self,
        name: &str,
        fields: &[(String, Type)],
        values: Vec<Detached>,
        expected: Option<DebugTypeId>,
    ) -> Result<Value, CodecError> {
        let id = self.record(name, expected)?;
        let metadata = self.image.records[usize::from(id.get())].fields.clone();
        if fields.len() != values.len() || metadata.len() != values.len() {
            return Err(invalid());
        }
        let mut output = Vec::new();
        for (((name, ty), value), field) in fields.iter().zip(values).zip(metadata) {
            if self.image.strings.get(field.name) != Some(name) {
                return Err(invalid());
            }
            output.push(self.pack(ty, value, Some(field.ty))?);
        }
        let layout = self
            .layouts
            .records
            .get(usize::from(id.get()))
            .cloned()
            .ok_or_else(invalid)?;
        Ok(Value::Record(SharedRecord::new(layout, output)))
    }

    fn text_record(
        &self,
        name: &str,
        field: &str,
        text: String,
        expected: Option<DebugTypeId>,
    ) -> Result<Value, CodecError> {
        let id = self.record(name, expected)?;
        let metadata = &self.image.records[usize::from(id.get())].fields;
        let [member] = metadata.as_slice() else {
            return Err(invalid());
        };
        if self.image.strings.get(member.name) != Some(field) {
            return Err(invalid());
        }
        self.scalar(Some(member.ty), |ty| matches!(ty, DebugType::String))?;
        let layout = self
            .layouts
            .records
            .get(usize::from(id.get()))
            .cloned()
            .ok_or_else(invalid)?;
        Ok(Value::Record(SharedRecord::new(
            layout,
            vec![Value::Str(text.into())],
        )))
    }

    fn enum_value(&self, index: usize, values: Vec<Value>) -> Result<Value, CodecError> {
        let layout = self
            .layouts
            .enum_variants
            .get(index)
            .cloned()
            .ok_or_else(invalid)?;
        SharedEnum::try_new(layout, values)
            .map(Value::Enum)
            .map_err(|_| invalid())
    }
}
