//! DAP custom-request mapping for dictionary structure mutation.

use serde_json::Value;

use super::DapServer;
use super::args;
use crate::engine::DebugOp;

impl DapServer {
    pub(super) fn insert_dictionary(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.mutating_request(
            request_seq,
            command,
            (|| {
                Ok(DebugOp::DictionaryInsert {
                    frame_id: args::optional_u64(arguments, "frameId")?,
                    target: args::required_string(arguments, "target")?,
                    key: args::required_string(arguments, "key")?,
                    expression: args::required_string(arguments, "value")?,
                })
            })(),
        )
    }

    pub(super) fn remove_dictionary(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.mutating_request(
            request_seq,
            command,
            (|| {
                Ok(DebugOp::DictionaryRemove {
                    frame_id: args::optional_u64(arguments, "frameId")?,
                    target: args::required_string(arguments, "target")?,
                    key: args::required_string(arguments, "key")?,
                })
            })(),
        )
    }

    pub(super) fn replace_dictionary_key(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.mutating_request(
            request_seq,
            command,
            (|| {
                Ok(DebugOp::DictionaryReplaceKey {
                    frame_id: args::optional_u64(arguments, "frameId")?,
                    target: args::required_string(arguments, "target")?,
                    key: args::required_string(arguments, "key")?,
                    new_key: args::required_string(arguments, "newKey")?,
                })
            })(),
        )
    }
}
