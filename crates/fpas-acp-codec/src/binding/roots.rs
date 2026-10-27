//! Typed payload conversion selected by SDK schema method metadata.

use super::{Catalog, Shape, Type, Value, Variant, invalid};
use crate::{
    CodecError, RawJson, codec,
    protocol::{v1, v2},
    schema::Version,
};
use agent_client_protocol::{JsonRpcMessage, JsonRpcResponse};
use serde::Serialize;

/// Typed payload accompanied by its exact received document.
pub struct DecodedPayload {
    /// Detached nominal root enum.
    pub payload: Value,
    /// Original JSON including ignored fields and arbitrary numeric spellings.
    pub raw: RawJson,
}

/// Whether an SDK root recognizes this method, including protocol extensions.
pub(super) fn supports_method(catalog: &Catalog, root: &str, method: &str) -> bool {
    method_variant(catalog, root, method).is_ok()
}

fn method_variant<'a>(
    catalog: &'a Catalog,
    root: &str,
    method: &str,
) -> Result<(&'a super::Declaration, &'a Variant, &'a super::Declaration), CodecError> {
    let owner = catalog.declarations.get(root).ok_or_else(invalid)?;
    let Shape::Enum { variants, .. } = &owner.shape else {
        return Err(invalid());
    };
    for variant in variants {
        let Some(Type::Named(key)) = &variant.payload else {
            continue;
        };
        let payload = catalog.declarations.get(key).ok_or_else(invalid)?;
        if payload.method.as_deref() == Some(method)
            || (matches!(payload.shape, Shape::Extension { .. })
                && method.starts_with('_')
                && method.len() > 1)
        {
            return Ok((owner, variant, payload));
        }
    }
    Err(invalid())
}

/// Decode a known request/response/notification family selected by its wire method.
/// SDK serde types validate each payload; no RPC envelope is sent here.
pub(super) fn decode_message(
    catalog: &Catalog,
    root: &str,
    method: &str,
    raw: RawJson,
) -> Result<DecodedPayload, CodecError> {
    let (owner, variant, declaration) = method_variant(catalog, root, method)?;
    let normalized =
        if raw.text() == "null" && !matches!(declaration.shape, Shape::Extension { .. }) {
            Some(sdk_normalize(catalog.version, root, method, &raw)?)
        } else {
            None
        };
    let source = normalized.as_ref().unwrap_or(&raw);
    let mut payload = super::decode::decode(catalog, &declaration.key, source)?;
    if matches!(declaration.shape, Shape::Extension { response: false }) {
        let Value::Record { fields, .. } = &mut payload else {
            return Err(invalid());
        };
        fields[0] = Value::String(method.into());
    }
    Ok(DecodedPayload {
        payload: Value::Enum {
            name: owner.name.clone(),
            variant: variant.name.clone(),
            fields: vec![payload],
        },
        raw,
    })
}

/// Encode a typed root only when its nominal family and method agree.
pub(super) fn encode_message(
    catalog: &Catalog,
    root: &str,
    method: &str,
    value: &Value,
) -> Result<RawJson, CodecError> {
    let (owner, variant, declaration) = method_variant(catalog, root, method)?;
    let Value::Enum {
        name,
        variant: actual,
        fields,
    } = value
    else {
        return Err(invalid());
    };
    if name != &owner.name || actual != &variant.name {
        return Err(invalid());
    }
    let [payload] = fields.as_slice() else {
        return Err(invalid());
    };
    if matches!(declaration.shape, Shape::Extension { response: false }) {
        let Value::Record { fields, .. } = payload else {
            return Err(invalid());
        };
        if !matches!(fields.first(), Some(Value::String(actual)) if actual == method) {
            return Err(invalid());
        }
    }
    super::encode::encode(catalog, &declaration.key, payload)
}

fn message<T: JsonRpcMessage + Serialize>(
    method: &str,
    raw: &RawJson,
) -> Result<RawJson, CodecError> {
    let value = T::parse_message(method, raw).map_err(|_| invalid())?;
    codec::encode(&value, crate::raw::MAX_RAW_BYTES)
}

fn response<T: JsonRpcResponse + Serialize>(
    method: &str,
    raw: &RawJson,
) -> Result<RawJson, CodecError> {
    let value = serde_json::from_str(raw.text()).map_err(CodecError::json)?;
    let value = T::from_value(method, value).map_err(|_| invalid())?;
    codec::encode(&value, crate::raw::MAX_RAW_BYTES)
}

fn sdk_normalize(
    version: Version,
    root: &str,
    method: &str,
    raw: &RawJson,
) -> Result<RawJson, CodecError> {
    match (version, root) {
        (Version::V1, "ClientRequest") => message::<v1::ClientRequest>(method, raw),
        (Version::V1, "AgentRequest") => message::<v1::AgentRequest>(method, raw),
        (Version::V1, "ClientNotification") => message::<v1::ClientNotification>(method, raw),
        (Version::V1, "AgentNotification") => message::<v1::AgentNotification>(method, raw),
        (Version::V1, "ProtocolLevelNotification") => {
            message::<v1::ProtocolLevelNotification>(method, raw)
        }
        (Version::V1, "AgentResponse") => response::<v1::AgentResponse>(method, raw),
        (Version::V1, "ClientResponse") => response::<v1::ClientResponse>(method, raw),
        (Version::V2, "ClientRequest") => message::<v2::ClientRequest>(method, raw),
        (Version::V2, "AgentRequest") => message::<v2::AgentRequest>(method, raw),
        (Version::V2, "ClientNotification") => message::<v2::ClientNotification>(method, raw),
        (Version::V2, "AgentNotification") => message::<v2::AgentNotification>(method, raw),
        (Version::V2, "ProtocolLevelNotification") => {
            message::<v2::ProtocolLevelNotification>(method, raw)
        }
        (Version::V2, "AgentResponse") => response::<v2::AgentResponse>(method, raw),
        (Version::V2, "ClientResponse") => response::<v2::ClientResponse>(method, raw),
        _ => Err(invalid()),
    }
}
