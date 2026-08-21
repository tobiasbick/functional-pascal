//! JSON identity objects shared by protocol adapters.

use serde_json::Value;

/// Parse a location identity object from adapter JSON.
#[must_use]
pub(crate) fn parse_identity(value: &Value) -> Option<fpas_vm::DebugDataLocationIdentity> {
    let object = value.as_object()?;
    if let Some(index) = object.get("index").and_then(Value::as_u64) {
        return Some(fpas_vm::DebugDataLocationIdentity::Global { index });
    }
    Some(fpas_vm::DebugDataLocationIdentity::FrameRegister {
        task_id: object.get("task_id").and_then(Value::as_u64)?,
        function: object.get("function").and_then(Value::as_u64)?,
        register: object.get("register").and_then(Value::as_u64)?,
    })
}
