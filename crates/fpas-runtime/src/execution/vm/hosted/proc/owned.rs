//! Native launch inputs never retain a Value, frame, or aggregate factory.

use super::options::{Options, Request};
use std::time::{Duration, Instant};

pub(super) struct OwnedRequest {
    pub command: String,
    arguments: Vec<String>,
    directory: Option<String>,
    environment: Vec<(String, Option<String>)>,
    input: String,
    timeout: Option<Duration>,
    maximum: usize,
    pub token: Option<u64>,
    pub capture: bool,
    inherit_input: bool,
    pub started: Instant,
}

impl OwnedRequest {
    pub(super) fn new(request: &Request<'_>) -> Self {
        let started = Instant::now();
        Self {
            command: request.command.to_owned(),
            arguments: request.arguments.iter().map(|s| (*s).to_owned()).collect(),
            directory: request.options.directory.map(str::to_owned),
            environment: request
                .options
                .environment
                .iter()
                .map(|(key, value)| ((*key).to_owned(), value.map(str::to_owned)))
                .collect(),
            input: request.options.input.to_owned(),
            timeout: request.options.timeout,
            maximum: request.options.maximum,
            token: request.options.token,
            capture: request.capture,
            inherit_input: request.inherit_input,
            started,
        }
    }

    pub(super) fn borrow(&self) -> Request<'_> {
        Request {
            command: &self.command,
            arguments: self.arguments.iter().map(String::as_str).collect(),
            options: Options {
                directory: self.directory.as_deref(),
                environment: self
                    .environment
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_deref()))
                    .collect(),
                input: &self.input,
                timeout: self.timeout,
                maximum: self.maximum,
                token: self.token,
            },
            capture: self.capture,
            inherit_input: self.inherit_input,
        }
    }
}
