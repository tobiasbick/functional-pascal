//! Translate the SDK JSON Schema graph into shared native declaration shapes.

mod fields;
mod unions;

use super::{Catalog, Declaration, ROOTS, Shape, Type, names::identifier};
use crate::{
    CodecError, ErrorKind,
    schema::{self, Version},
};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn invalid() -> CodecError {
    CodecError::new(ErrorKind::InvalidMessage)
}

pub(super) struct Builder {
    pub version: Version,
    pub modules: BTreeMap<String, &'static str>,
    pub declarations: BTreeMap<String, Declaration>,
}

pub(super) fn catalog(version: Version) -> Result<Catalog, CodecError> {
    let document = schema::document(version);
    let mut schemas = document["$defs"]
        .as_object()
        .ok_or_else(invalid)?
        .iter()
        .map(|(name, value)| (name.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let roots = document["prefixItems"].as_array().ok_or_else(invalid)?;
    for (index, name) in ROOTS.iter().enumerate() {
        let index = if index == 6 { 9 } else { index };
        schemas.insert((*name).into(), roots.get(index).ok_or_else(invalid)?);
    }
    let mut stable = BTreeSet::new();
    for name in ROOTS {
        mark_stable(
            &schemas,
            &Value::Object(Map::from_iter([(
                "$ref".into(),
                Value::String(format!("#/$defs/{name}")),
            )])),
            &mut stable,
        );
    }
    let modules = schemas
        .keys()
        .map(|name| {
            (
                name.clone(),
                if version == Version::V2 {
                    "Std.Ai.Acp.V2"
                } else if stable.contains(name) {
                    "Std.Ai.Acp.Types"
                } else {
                    "Std.Ai.Acp.Experimental"
                },
            )
        })
        .collect();
    let mut builder = Builder {
        version,
        modules,
        declarations: BTreeMap::new(),
    };
    for (key, schema) in schemas {
        builder.declare(&key, schema)?;
    }
    Ok(Catalog {
        version,
        declarations: builder.declarations,
    })
}

fn mark_stable(schemas: &BTreeMap<String, &Value>, schema: &Value, seen: &mut BTreeSet<String>) {
    if schema["description"]
        .as_str()
        .is_some_and(|text| text.starts_with("**UNSTABLE**"))
    {
        return;
    }
    if let Some(name) = reference(schema)
        && let Some(target) = schemas.get(name)
        && !target["description"]
            .as_str()
            .is_some_and(|text| text.starts_with("**UNSTABLE**"))
        && seen.insert(name.into())
    {
        mark_stable(schemas, target, seen);
    }
    match schema {
        Value::Object(fields) => {
            for value in fields.values() {
                mark_stable(schemas, value, seen);
            }
        }
        Value::Array(items) => {
            for value in items {
                mark_stable(schemas, value, seen);
            }
        }
        _ => {}
    }
}

pub(super) fn reference(schema: &Value) -> Option<&str> {
    schema["$ref"].as_str()?.strip_prefix("#/$defs/")
}

impl Builder {
    pub fn module(&self, owner: &str) -> &'static str {
        self.modules
            .iter()
            .filter(|(name, _)| owner.starts_with(name.as_str()))
            .max_by_key(|(name, _)| name.len())
            .map(|(_, module)| *module)
            .unwrap_or(if self.version == Version::V1 {
                "Std.Ai.Acp.Types"
            } else {
                "Std.Ai.Acp.V2"
            })
    }
    pub fn declare(&mut self, key: &str, schema: &Value) -> Result<(), CodecError> {
        if self.declarations.contains_key(key) {
            return Ok(());
        }
        let module = self.module(key);
        let name = if key == "Error" {
            "Std.Ai.Acp.Types.ProtocolError".into()
        } else {
            format!("{module}.{}", identifier(key))
        };
        let shape = if matches!(key, "ExtRequest" | "ExtNotification" | "ExtResponse") {
            Shape::Extension {
                response: key == "ExtResponse",
            }
        } else if schema.get("properties").is_some() {
            Shape::Record(self.fields(key, schema)?)
        } else if schema
            .get("oneOf")
            .or_else(|| schema.get("anyOf"))
            .is_some()
            || schema.get("enum").is_some()
            || schema.get("const").is_some()
        {
            self.union(key, schema)?
        } else {
            Shape::Scalar(self.ty(key, schema)?)
        };
        self.declarations.insert(
            key.into(),
            Declaration {
                key: key.into(),
                name,
                module,
                shape,
                method: schema["x-method"].as_str().map(str::to_owned),
            },
        );
        Ok(())
    }

    pub fn ty(&mut self, owner: &str, schema: &Value) -> Result<Type, CodecError> {
        if let Some(name) = reference(schema) {
            return Ok(Type::Named(name.into()));
        }
        if let Some(types) = schema["type"].as_array() {
            let nonnull = types
                .iter()
                .filter(|ty| ty.as_str() != Some("null"))
                .collect::<Vec<_>>();
            if nonnull.len() == 1 && nonnull.len() < types.len() {
                let mut normalized = schema.clone();
                normalized["type"] = nonnull[0].clone();
                return Ok(Type::Option(Box::new(self.ty(owner, &normalized)?)));
            }
        } else if let Some(branches) = schema["anyOf"].as_array()
            && branches.len() == 2
            && branches.iter().any(|s| s["type"] == "null")
        {
            let branch = branches
                .iter()
                .find(|s| s["type"] != "null")
                .ok_or_else(invalid)?;
            return Ok(Type::Option(Box::new(self.ty(owner, branch)?)));
        }
        if schema
            .get("oneOf")
            .or_else(|| schema.get("anyOf"))
            .is_some()
            || schema.get("enum").is_some()
            || schema.get("const").is_some()
            || schema.get("properties").is_some()
        {
            let key = format!("{owner}Value");
            let module = self.module(owner);
            self.modules.entry(key.clone()).or_insert(module);
            self.declare(&key, schema)?;
            return Ok(Type::Named(key));
        }
        Ok(match schema["type"].as_str() {
            Some("boolean") => Type::Boolean,
            Some("integer") if schema["format"] == "uint64" => Type::UInt64,
            Some("integer") => {
                let (minimum, maximum) = match schema["format"].as_str() {
                    Some("uint8") => (0, i64::from(u8::MAX)),
                    Some("uint16") => (0, i64::from(u16::MAX)),
                    Some("uint32") => (0, i64::from(u32::MAX)),
                    Some("int8") => (i64::from(i8::MIN), i64::from(i8::MAX)),
                    Some("int16") => (i64::from(i16::MIN), i64::from(i16::MAX)),
                    Some("int32") => (i64::from(i32::MIN), i64::from(i32::MAX)),
                    _ => (i64::MIN, i64::MAX),
                };
                Type::Integer {
                    minimum: schema["minimum"].as_i64().unwrap_or(minimum),
                    maximum: schema["maximum"].as_i64().unwrap_or(maximum),
                }
            }
            Some("number") => Type::Real,
            Some("string") => Type::String,
            Some("array") => Type::Array(Box::new(
                self.ty(&format!("{owner}Item"), &schema["items"])?,
            )),
            Some("object") => {
                if schema["additionalProperties"].is_object() {
                    Type::Dictionary(Box::new(
                        self.ty(&format!("{owner}Entry"), &schema["additionalProperties"])?,
                    ))
                } else {
                    Type::Raw { object: true }
                }
            }
            None if schema
                .as_object()
                .is_some_and(|schema| !schema.contains_key("type"))
                || *schema == true =>
            {
                Type::Raw { object: false }
            }
            _ => return Err(invalid()),
        })
    }
}
