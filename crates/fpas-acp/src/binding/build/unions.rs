//! Tagged, untagged and open SDK enums, including flattened payloads.

use super::{Builder, invalid, reference};
use crate::{
    CodecError,
    binding::{Shape, Type, Unknown, Variant, names::identifier},
};
use serde_json::Value;

impl Builder {
    pub(super) fn union(&mut self, owner: &str, schema: &Value) -> Result<Shape, CodecError> {
        let scalar_literals = schema
            .get("enum")
            .and_then(Value::as_array)
            .cloned()
            .or_else(|| schema.get("const").map(|value| vec![value.clone()]));
        let branches = scalar_literals
            .map(|literals| {
                literals
                    .into_iter()
                    .map(|value| serde_json::json!({"const": value}))
                    .collect::<Vec<_>>()
            })
            .or_else(|| {
                schema
                    .get("oneOf")
                    .or_else(|| schema.get("anyOf"))
                    .and_then(Value::as_array)
                    .cloned()
            })
            .ok_or_else(invalid)?;
        let discriminator = branches.iter().find_map(tag).and_then(|(name, _)| name);
        let open_string = branches
            .iter()
            .all(|branch| branch["type"] == "string" || branch["const"].is_string());
        let mut variants = Vec::new();
        let mut unknown = None;
        for (index, branch) in branches.iter().enumerate() {
            if is_unknown(branch, discriminator.as_deref(), open_string) {
                unknown = Some(if let Some(tag) = &discriminator {
                    Unknown::Tagged(tag.clone())
                } else {
                    Unknown::String
                });
                continue;
            }
            let fallback = crate::binding::names::default_variant(self.version, owner, index);
            let tag = fallback
                .map(|(_, value)| (Some("type".into()), Value::String(value.into())))
                .or_else(|| tag(branch));
            let default_tag = fallback.is_some()
                || tag.as_ref().is_some_and(|(property, expected)| {
                    property.as_ref().is_some_and(|property| {
                        branch["properties"][property]["default"] == *expected
                            && !branch["required"].as_array().is_some_and(|required| {
                                required.iter().any(|name| name == property)
                            })
                    })
                });
            let literal = branch.get("const");
            let name = if let Some((name, _)) = fallback {
                name.into()
            } else if let Some((_, Value::String(tag))) = &tag {
                identifier(tag)
            } else if let Some(title) = branch["title"].as_str() {
                identifier(title)
            } else if let Some(reference) = reference(branch) {
                identifier(reference)
            } else if branch["type"] == "null" {
                "NullValue".into()
            } else if branch["type"] == "array" {
                reference(&branch["items"]).map_or_else(
                    || format!("Items{}", index + 1),
                    |name| format!("{}Items", identifier(name)),
                )
            } else if let Some(kind) = branch["type"].as_str().filter(|kind| *kind != "object") {
                identifier(kind)
            } else if let Some(value) = literal {
                identifier(value.as_str().unwrap_or("Literal"))
            } else {
                format!("Variant{}", index + 1)
            };
            let payload = if literal.is_some()
                || (tag.as_ref().is_some_and(|(name, _)| name.is_none()))
                || branch["type"] == "null"
            {
                None
            } else if let Some(reference) = reference(branch) {
                Some(Type::Named(reference.into()))
            } else {
                let mut body = branch.clone();
                if let Some((Some(tag), _)) = &tag {
                    if let Some(properties) = body["properties"].as_object_mut() {
                        properties.remove(tag);
                    }
                    if let Some(required) = body["required"].as_array_mut() {
                        required.retain(|name| name != tag);
                    }
                }
                if body["properties"]
                    .as_object()
                    .is_some_and(|properties| properties.is_empty())
                {
                    None
                } else {
                    Some(self.ty(&format!("{owner}{name}"), &body)?)
                }
            };
            variants.push(Variant {
                name,
                tag,
                default_tag,
                payload,
            });
        }
        if let Some(discriminator) = discriminator {
            unknown = Some(Unknown::Tagged(discriminator));
        } else if variants
            .iter()
            .all(|variant| matches!(variant.tag, Some((None, Value::String(_)))))
        {
            unknown = Some(Unknown::String);
        }
        let mut names = std::collections::HashSet::new();
        for variant in &mut variants {
            if !names.insert(variant.name.to_ascii_lowercase()) {
                variant.name.push_str(&format!("{}", names.len()));
                if !names.insert(variant.name.to_ascii_lowercase()) {
                    return Err(invalid());
                }
            }
        }
        Ok(Shape::Enum { variants, unknown })
    }
}

pub(super) fn tag(schema: &Value) -> Option<(Option<String>, Value)> {
    if let Some(value) = schema.get("const") {
        return Some((None, value.clone()));
    }
    if schema["type"] == "null" {
        return Some((None, Value::Null));
    }
    schema["properties"]
        .as_object()?
        .iter()
        .find_map(|(name, value)| {
            value
                .get("const")
                .cloned()
                .or_else(|| {
                    value["enum"]
                        .as_array()
                        .filter(|values| values.len() == 1)
                        .map(|values| values[0].clone())
                })
                .or_else(|| (value["type"] == "null").then_some(Value::Null))
                .map(|value| (Some(name.clone()), value))
        })
}

fn is_unknown(schema: &Value, discriminator: Option<&str>, open_string: bool) -> bool {
    if open_string && schema["type"] == "string" && schema.get("const").is_none() {
        return true;
    }
    if let Some(tag) = discriminator {
        return schema["properties"].get(tag).is_some_and(|field| {
            field["type"] == "string" && field.get("const").is_none() && field.get("enum").is_none()
        }) && schema["additionalProperties"] != false;
    }
    false
}
