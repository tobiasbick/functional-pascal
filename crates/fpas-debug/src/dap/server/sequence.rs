//! DAP custom-request mapping for sequence structure mutation.

use serde_json::Value;

use super::DapServer;
use super::args;
use crate::engine::DebugOp;

impl DapServer {
    pub(super) fn insert_array(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.mutating_request(
            request_seq,
            command,
            (|| {
                Ok(DebugOp::ArrayInsert {
                    frame_id: args::optional_u64(arguments, "frameId")?,
                    target: args::required_string(arguments, "target")?,
                    index: args::required_string(arguments, "index")?,
                    expression: args::required_string(arguments, "value")?,
                })
            })(),
        )
    }

    pub(super) fn remove_array(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.mutating_request(
            request_seq,
            command,
            (|| {
                Ok(DebugOp::ArrayRemove {
                    frame_id: args::optional_u64(arguments, "frameId")?,
                    target: args::required_string(arguments, "target")?,
                    index: args::required_string(arguments, "index")?,
                })
            })(),
        )
    }

    pub(super) fn replace_string_character(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.mutating_request(
            request_seq,
            command,
            (|| {
                Ok(DebugOp::StringReplaceCharacter {
                    frame_id: args::optional_u64(arguments, "frameId")?,
                    target: args::required_string(arguments, "target")?,
                    index: args::required_string(arguments, "index")?,
                    expression: args::required_string(arguments, "value")?,
                })
            })(),
        )
    }
}
