//! Ordered record output with explicit option, patch and extra-member handling.

use super::{Encoder, invalid};
use crate::{
    CodecError,
    binding::wire::Wire,
    binding::{Declaration, Field, Value, WireField},
};
use std::collections::HashSet;

impl Encoder<'_> {
    pub(super) fn record(
        &mut self,
        declaration: &Declaration,
        fields: &[Field],
        value: &Value,
    ) -> Result<Wire, CodecError> {
        let Value::Record {
            name,
            fields: values,
        } = value
        else {
            return Err(invalid());
        };
        if name != &declaration.name || fields.len() != values.len() {
            return Err(invalid());
        }
        let mut output = Vec::new();
        let mut occupied = HashSet::new();
        for field in fields {
            match &field.wire {
                WireField::Property(name) => {
                    occupied.insert(name.clone());
                }
                WireField::Flatten => self.catalog.wire_properties(&field.ty, &mut occupied),
                WireField::Extra => {}
            }
        }
        for (field, value) in fields.iter().zip(values) {
            if !field.required
                && matches!(&field.wire, WireField::Property(_))
                && matches!(value, Value::None | Value::Missing)
            {
                if !matches!(
                    field.ty,
                    crate::binding::Type::Option(_) | crate::binding::Type::Field(_)
                ) {
                    return Err(invalid());
                }
                continue;
            }
            let wire = self.ty(&field.ty, value)?;
            match &field.wire {
                WireField::Property(name) => output.push((name.clone(), wire)),
                WireField::Flatten => output.extend(wire.object()?),
                WireField::Extra => {
                    for (name, value) in wire.object()? {
                        if occupied.contains(&name) {
                            return Err(invalid());
                        }
                        output.push((name, value));
                    }
                }
            }
        }
        Ok(Wire::Object(output))
    }
}
