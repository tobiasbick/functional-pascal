use crate::{RawJson, connection::Fault};
use serde_json::Value;

fn json(raw: &RawJson) -> Result<Value, Fault> {
    serde_json::from_str(raw.text()).map_err(|_| Fault::InvalidMessage)
}
fn invalid() -> Fault {
    Fault::InvalidArgument("ElicitationContent")
}

/// Build editable initial form values. This never accepts or sends an interaction.
pub fn defaults(request: &RawJson) -> Result<RawJson, Fault> {
    let request = json(request)?;
    let properties = request
        .pointer("/requestedSchema/properties")
        .and_then(Value::as_object)
        .ok_or(Fault::InvalidArgument("ElicitationMode"))?;
    let values: serde_json::Map<String, Value> = properties
        .iter()
        .filter_map(|(key, schema)| {
            schema
                .get("default")
                .map(|value| (key.clone(), value.clone()))
        })
        .collect();
    RawJson::parse(&Value::Object(values).to_string()).map_err(Into::into)
}

pub(super) fn validate(request: &RawJson, response: &RawJson) -> Result<(), Fault> {
    let request = json(request)?;
    let response = json(response)?;
    if response["action"] != "accept" {
        return Ok(());
    }
    if request["mode"] == "url" {
        if response
            .get("content")
            .is_some_and(|value| !value.is_null())
        {
            return Err(invalid());
        }
        return Ok(());
    }
    let schema = &request["requestedSchema"];
    let properties = schema["properties"].as_object().ok_or_else(invalid)?;
    let values = response["content"].as_object().ok_or_else(invalid)?;
    if let Some(required) = schema["required"].as_array() {
        for key in required {
            if !values.contains_key(key.as_str().ok_or_else(invalid)?) {
                return Err(invalid());
            }
        }
    }
    for (key, value) in values {
        let field = properties.get(key).ok_or_else(invalid)?;
        let valid = match field["type"].as_str() {
            Some("boolean") => value.is_boolean(),
            Some("integer") => value.as_i64().is_some_and(|number| {
                field["minimum"]
                    .as_i64()
                    .is_none_or(|minimum| number >= minimum)
                    && field["maximum"]
                        .as_i64()
                        .is_none_or(|maximum| number <= maximum)
            }),
            Some("number") => value.as_f64().is_some_and(|number| {
                number.is_finite()
                    && field["minimum"]
                        .as_f64()
                        .is_none_or(|minimum| number >= minimum)
                    && field["maximum"]
                        .as_f64()
                        .is_none_or(|maximum| number <= maximum)
            }),
            Some("string") => {
                if let Some(text) = value.as_str() {
                    string(field, text)?
                } else {
                    false
                }
            }
            Some("array") => {
                if let Some(values) = value.as_array() {
                    field["minItems"]
                        .as_u64()
                        .is_none_or(|min| values.len() as u64 >= min)
                        && field["maxItems"]
                            .as_u64()
                            .is_none_or(|max| values.len() as u64 <= max)
                        && values
                            .iter()
                            .all(|value| value.as_str().is_some() && choice(&field["items"], value))
                        && (field["uniqueItems"] != true
                            || values
                                .iter()
                                .enumerate()
                                .all(|(index, value)| !values[..index].contains(value)))
                } else {
                    false
                }
            }
            _ => false,
        };
        if !valid {
            return Err(invalid());
        }
    }
    Ok(())
}

fn choice(schema: &Value, value: &Value) -> bool {
    schema["enum"]
        .as_array()
        .is_none_or(|choices| choices.contains(value))
        && schema["oneOf"]
            .as_array()
            .is_none_or(|choices| choices.iter().any(|choice| &choice["const"] == value))
        && schema["anyOf"]
            .as_array()
            .is_none_or(|choices| choices.iter().any(|choice| &choice["const"] == value))
}
fn string(schema: &Value, text: &str) -> Result<bool, Fault> {
    let length = text.chars().count() as u64;
    if schema["minLength"].as_u64().is_some_and(|min| length < min)
        || schema["maxLength"].as_u64().is_some_and(|max| length > max)
        || !choice(schema, &Value::String(text.into()))
    {
        return Ok(false);
    }
    if let Some(pattern) = schema["pattern"].as_str() {
        if pattern.len() > 4096 {
            return Err(Fault::LimitExceeded("ElicitationPatternBytes", 4096));
        }
        let regex = fancy_regex::RegexBuilder::new(pattern)
            .backtrack_limit(100_000)
            .delegate_size_limit(1024 * 1024)
            .build()
            .map_err(|_| Fault::InvalidArgument("ElicitationPattern"))?;
        return regex
            .is_match(text)
            .map_err(|_| Fault::LimitExceeded("ElicitationPatternSteps", 100_000));
    }
    Ok(true)
}
