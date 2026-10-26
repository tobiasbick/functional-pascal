//! Inactive response-only extensions remain in RawMessage without activating typed fields.
use super::v1::Client;
use crate::{
    binding::{self, Shape, Value},
    schema::Version,
};

impl Client {
    pub fn filter_usage(&self, value: &mut Value, version: Version) {
        if !self.settings.experimental.token_usage
            && let Ok(catalog) = binding::catalog(version)
        {
            filter(value, catalog);
        }
    }
}
fn filter(value: &mut Value, catalog: &binding::Catalog) {
    match value {
        Value::Record { name, fields } => {
            let key = name.rsplit('.').next().unwrap_or("");
            if matches!(key, "PromptResponse" | "IdleStateUpdate")
                && let Some(declaration) = catalog.declarations.get(key)
                && let Shape::Record(declarations) = &declaration.shape
            {
                for (field, value) in declarations.iter().zip(fields.iter_mut()) {
                    if field.name == "Usage" {
                        *value = Value::None;
                    }
                }
            }
            for value in fields {
                filter(value, catalog);
            }
        }
        Value::Enum { fields, .. } | Value::Array(fields) => {
            for value in fields {
                filter(value, catalog);
            }
        }
        Value::Some(value) | Value::Replacement(value) => filter(value, catalog),
        Value::Dictionary(fields) => {
            for (_, value) in fields {
                filter(value, catalog);
            }
        }
        _ => {}
    }
}
