//! DAP custom-request mapping for variant discovery and construction.

use serde_json::{Value, json};

use super::DapServer;
use super::args;
use super::values;
use crate::engine::{DebugOp, ResponseBody};

impl DapServer {
    pub(super) fn describe_variant(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        let op: Result<DebugOp, String> = (|| {
            Ok(DebugOp::VariantDescribe {
                frame_id: args::optional_u64(arguments, "frameId")?,
                target: args::required_string(arguments, "target")?,
            })
        })();
        match op {
            Ok(op) => self.core_request(request_seq, command, op),
            Err(message) => vec![self.failure(request_seq, command, &message)],
        }
    }

    pub(super) fn construct_variant(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.mutating_request(
            request_seq,
            command,
            (|| {
                Ok(DebugOp::VariantConstruct {
                    frame_id: args::optional_u64(arguments, "frameId")?,
                    target: args::required_string(arguments, "target")?,
                    variant: args::required_string(arguments, "variant")?,
                    fields: args::parse_variant_fields(arguments)?,
                })
            })(),
        )
    }
}

/// Translate one variant custom-request result into DAP naming.
pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    match (command, body) {
        (
            "fpas/variantDescribe",
            ResponseBody::VariantDescription {
                target,
                description,
            },
        ) => Some(json!({
            "target": target,
            "typeName": description.type_name,
            "variants": description.variants.iter().map(|variant| {
                json!({
                    "name": variant.name,
                    "fields": variant.fields.iter().map(|field| {
                        json!({
                            "name": field.name,
                            "typeName": field.type_name
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        })),
        ("fpas/variantConstruct", ResponseBody::VariantConstruct(result)) => {
            let mut mapped = values::variable_value_json(&result.value);
            mapped.insert("variant".into(), Value::String(result.variant.clone()));
            Some(Value::Object(mapped))
        }
        _ => None,
    }
}
