//! Application-controlled permission and elicitation validation; no host effects.
mod elicitation;
mod permission;
pub use elicitation::defaults;

use crate::{RawJson, connection::Fault};

/// Validate a reply against the actual incoming interaction before publishing it.
pub(crate) fn validate(method: &str, request: &RawJson, response: &RawJson) -> Result<(), Fault> {
    match method {
        "session/request_permission" => permission::validate(request, response),
        "elicitation/create" => elicitation::validate(request, response),
        _ => Ok(()),
    }
}
