//! One catalog pairs SDK definitions with application handler identities.

use crate::{
    Fault, Limits,
    model::{CallToolResult, JsonObject, Tool},
};
use jsonschema::Validator;
use serde_json::Value;
use std::collections::BTreeMap;

struct Entry {
    tool: Tool,
    handler: u64,
    input: Validator,
    output: Option<Validator>,
}

/// Mutable registration builder, consumed when publishing a server.
/// Handler identities refer to callbacks owned by the embedding application.
pub struct Catalog {
    pub(crate) limits: Limits,
    entries: BTreeMap<String, Entry>,
    bytes: usize,
}

impl Catalog {
    /// Validate all resource limits before accepting tool definitions.
    pub fn new(limits: Limits) -> Result<Self, Fault> {
        limits.validate()?;
        Ok(Self {
            limits,
            entries: BTreeMap::new(),
            bytes: 2,
        })
    }

    /// Register a definition once, compiling schemas without file or network access.
    /// Failure leaves the catalog unchanged.
    pub fn register(&mut self, tool: Tool, handler: u64) -> Result<(), Fault> {
        if tool.name.is_empty()
            || tool.name.len() > 128
            || !tool
                .name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
        {
            return Err(Fault::InvalidArgument("ToolName"));
        }
        if self.entries.contains_key(tool.name.as_ref()) {
            return Err(Fault::DuplicateTool);
        }
        if self.entries.len() == self.limits.tools {
            return Err(Fault::LimitExceeded("Tools", self.limits.tools));
        }
        let bytes = serde_json::to_vec(&tool)
            .map_err(|_| Fault::InvalidArgument("Tool"))?
            .len()
            .saturating_add(usize::from(!self.entries.is_empty()));
        if bytes.saturating_add(self.bytes) > self.limits.catalog_bytes {
            return Err(Fault::LimitExceeded(
                "CatalogBytes",
                self.limits.catalog_bytes,
            ));
        }
        if tool.input_schema.get("type") != Some(&Value::String("object".into())) {
            return Err(Fault::InvalidSchema("InputSchema"));
        }
        let input = compile(&tool.input_schema, "InputSchema")?;
        let output = tool
            .output_schema
            .as_ref()
            .map(|schema| compile(schema, "OutputSchema"))
            .transpose()?;
        self.entries.insert(
            tool.name.to_string(),
            Entry {
                tool,
                handler,
                input,
                output,
            },
        );
        self.bytes += bytes;
        Ok(())
    }

    /// Registered definitions in deterministic name order.
    pub fn tools(&self) -> impl Iterator<Item = &Tool> {
        self.entries.values().map(|entry| &entry.tool)
    }

    /// The definition used for both discovery and invocation.
    pub fn tool(&self, name: &str) -> Option<&Tool> {
        self.entries.get(name).map(|entry| &entry.tool)
    }

    pub(crate) fn validate_arguments(
        &self,
        name: &str,
        arguments: &JsonObject,
    ) -> Result<u64, Fault> {
        let entry = self.entries.get(name).ok_or(Fault::UnknownTool)?;
        if !entry.input.is_valid(&Value::Object(arguments.clone())) {
            return Err(Fault::InvalidArguments);
        }
        Ok(entry.handler)
    }

    pub(crate) fn validate_result(&self, name: &str, result: &CallToolResult) -> Result<(), Fault> {
        let entry = self.entries.get(name).ok_or(Fault::UnknownTool)?;
        if result.is_error != Some(true)
            && let Some(schema) = &entry.output
        {
            let value = result
                .structured_content
                .as_ref()
                .ok_or(Fault::InvalidResult)?;
            if !schema.is_valid(value) {
                return Err(Fault::InvalidResult);
            }
        }
        Ok(())
    }
}

fn compile(schema: &JsonObject, name: &'static str) -> Result<Validator, Fault> {
    jsonschema::options()
        .offline()
        .build(&Value::Object(schema.clone()))
        .map_err(|_| Fault::InvalidSchema(name))
}
