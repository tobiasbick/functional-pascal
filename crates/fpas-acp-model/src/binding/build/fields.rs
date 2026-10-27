//! Record fields, flattened unions and SDK presence/default rules.

use super::{Builder, invalid};
use crate::{
    CodecError,
    binding::{Field, Type, WireField, names},
};
use serde_json::Value;

impl Builder {
    pub(super) fn fields(&mut self, owner: &str, schema: &Value) -> Result<Vec<Field>, CodecError> {
        let mut fields = Vec::new();
        let discriminator = schema
            .get("oneOf")
            .or_else(|| schema.get("anyOf"))
            .and_then(Value::as_array)
            .and_then(|branches| branches.iter().find_map(super::unions::tag))
            .and_then(|(name, _)| name);
        let properties = schema["properties"].as_object().ok_or_else(invalid)?;
        for (wire, property) in properties {
            if discriminator.as_ref() == Some(wire) {
                continue;
            }
            let required = schema["required"]
                .as_array()
                .is_some_and(|names| names.iter().any(|name| name == wire));
            let mut ty = self.ty(&format!("{owner}{}", names::identifier(wire)), property)?;
            if owner == "Error" && wire == "code" {
                ty = Type::Integer {
                    minimum: i64::from(i32::MIN),
                    maximum: i64::from(i32::MAX),
                };
            }
            if names::patch(self.version, owner, wire) {
                let inner = if let Type::Option(inner) = ty {
                    *inner
                } else {
                    ty
                };
                ty = Type::Field(Box::new(inner));
            } else if !required
                && property.get("default").is_none()
                && !matches!(ty, Type::Option(_))
            {
                ty = Type::Option(Box::new(ty));
            }
            fields.push(Field {
                name: names::identifier(wire),
                wire: WireField::Property(wire.clone()),
                ty,
                required,
                default: property.get("default").cloned(),
                default_on_error: property["x-deserialize-default-on-error"] == true,
                skip_invalid_items: property["x-deserialize-skip-invalid-items"] == true,
            });
        }
        if schema
            .get("oneOf")
            .or_else(|| schema.get("anyOf"))
            .is_some()
        {
            let key = format!("{owner}Kind");
            let mut union = schema.clone();
            union
                .as_object_mut()
                .ok_or_else(invalid)?
                .remove("properties");
            union
                .as_object_mut()
                .ok_or_else(invalid)?
                .remove("required");
            self.modules.insert(key.clone(), self.module(owner));
            let shape = self.union(&key, &union)?;
            self.declarations.insert(
                key.clone(),
                crate::binding::Declaration {
                    name: format!("{}.{}", self.module(owner), names::identifier(&key)),
                    key: key.clone(),
                    module: self.module(owner),
                    shape,
                    method: None,
                },
            );
            let tolerant = discriminator.as_ref().is_some_and(|name| {
                schema["properties"][name]["x-deserialize-default-on-error"] == true
            });
            fields.push(Field {
                name: "Kind".into(),
                wire: WireField::Flatten,
                ty: Type::Named(key),
                required: !tolerant,
                default: tolerant.then(|| serde_json::json!({})),
                default_on_error: tolerant,
                skip_invalid_items: false,
            });
        }
        if schema["additionalProperties"] == true {
            fields.push(Field {
                name: "Extra".into(),
                wire: WireField::Extra,
                ty: Type::Raw { object: true },
                required: false,
                default: Some(serde_json::json!({})),
                default_on_error: false,
                skip_invalid_items: false,
            });
        }
        let mut names = std::collections::HashSet::new();
        if fields
            .iter()
            .any(|field| !names.insert(field.name.to_ascii_lowercase()))
        {
            return Err(invalid());
        }
        Ok(fields)
    }
}
