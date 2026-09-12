//! DAP reverse request for an extension-owned external terminal frontend.

use serde_json::{Map, Value, json};

use super::DapServer;

const CONFIGURATION_FIELD: &str = "__fpasExternalTerminal";

impl DapServer {
    pub(super) fn launch(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.stop_on_entry = arguments
            .get("stopOnEntry")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let Some(configuration) = arguments.get(CONFIGURATION_FIELD) else {
            return vec![self.success(request_seq, command, json!({}))];
        };
        if !self.supports_run_in_terminal_request {
            return vec![self.failure(
                request_seq,
                command,
                "The debug client cannot open an external terminal. Select integratedTerminal or use a client that supports the DAP runInTerminal request.",
            )];
        }
        match self.run_in_terminal_request(configuration) {
            Ok((reverse_seq, reverse_request)) => {
                self.pending_run_in_terminal_request = Some((reverse_seq, request_seq));
                vec![reverse_request]
            }
            Err(message) => vec![self.failure(request_seq, command, &message)],
        }
    }

    pub(super) fn handle_client_response(&mut self, response: &Value) -> Vec<Value> {
        let request_seq = response
            .get("request_seq")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let command = response
            .get("command")
            .and_then(Value::as_str)
            .unwrap_or("<missing>");
        let Some((reverse_seq, launch_seq)) = self.pending_run_in_terminal_request else {
            return vec![self.unexpected_client_response()];
        };
        if reverse_seq != request_seq || command != "runInTerminal" {
            return vec![self.unexpected_client_response()];
        }
        self.pending_run_in_terminal_request = None;
        if response
            .get("success")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return vec![self.success(launch_seq, "launch", json!({}))];
        }
        let message = response
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("the debug client rejected the request");
        vec![self.failure(
            launch_seq,
            "launch",
            &format!("Cannot open the external FPAS terminal: {message}"),
        )]
    }

    fn run_in_terminal_request(&mut self, configuration: &Value) -> Result<(u64, Value), String> {
        let object = configuration
            .as_object()
            .ok_or_else(invalid_configuration)?;
        let title = required_string(object, "title")?;
        let cwd = required_string(object, "cwd")?;
        let args = required_string_array(object, "args")?;
        let env = required_string_map(object, "env")?;
        let seq = self.take_seq();
        Ok((
            seq,
            json!({
                "seq": seq,
                "type": "request",
                "command": "runInTerminal",
                "arguments": {
                    "kind": "external",
                    "title": title,
                    "cwd": cwd,
                    "args": args,
                    "env": env
                }
            }),
        ))
    }

    fn unexpected_client_response(&mut self) -> Value {
        self.event(
            "output",
            json!({
                "category": "stderr",
                "output": "The debug client sent an unexpected DAP response.\n"
            }),
        )
    }
}

fn required_string<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(invalid_configuration)
}

fn required_string_array(object: &Map<String, Value>, field: &str) -> Result<Vec<String>, String> {
    let values = object
        .get(field)
        .and_then(Value::as_array)
        .filter(|values| !values.is_empty())
        .ok_or_else(invalid_configuration)?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(invalid_configuration)
        })
        .collect()
}

fn required_string_map(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Map<String, Value>, String> {
    let values = object
        .get(field)
        .and_then(Value::as_object)
        .ok_or_else(invalid_configuration)?;
    if values.values().any(|value| !value.is_string()) {
        return Err(invalid_configuration());
    }
    Ok(values.clone())
}

fn invalid_configuration() -> String {
    "The external terminal launch data is invalid. Reinstall or update the Functional Pascal extension."
        .to_string()
}
