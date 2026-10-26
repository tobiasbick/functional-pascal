use crate::{RawJson, connection::Fault};

pub(super) fn validate(request: &RawJson, response: &RawJson) -> Result<(), Fault> {
    let request: serde_json::Value =
        serde_json::from_str(request.text()).map_err(|_| Fault::InvalidMessage)?;
    let response: serde_json::Value =
        serde_json::from_str(response.text()).map_err(|_| Fault::InvalidMessage)?;
    if response["outcome"]["outcome"] == "selected"
        && !request["options"].as_array().is_some_and(|options| {
            options
                .iter()
                .any(|option| option["optionId"] == response["outcome"]["optionId"])
        })
    {
        return Err(Fault::InvalidArgument("PermissionOptionId"));
    }
    Ok(())
}
