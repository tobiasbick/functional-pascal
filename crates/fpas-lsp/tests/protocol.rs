//! Protocol tests for LSP initialization, shutdown, and request validation.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol fixtures use explicit panics to keep hard-coded transcript failures local"
)]

mod support;

use fpas_language_service::DocumentStore;
use fpas_lsp::convert::{
    PositionConversionError, byte_offset_to_position, file_uri_to_path, position_to_byte_offset,
};
use tower_lsp_server::ls_types::{Position, Uri};

use support::{exit, initialize, initialized, response, run, shutdown};

fn snapshot(source: &str) -> std::sync::Arc<fpas_language_service::DocumentSnapshot> {
    DocumentStore::new()
        .open_document(std::path::Path::new("phase4-position-test.fpas"), 1, source)
        .expect("create in-memory snapshot")
}

#[test]
fn positions_convert_ascii_crlf_unicode_and_document_boundaries() {
    let snapshot = snapshot("abc\r\né𝄞z\n");

    assert_eq!(
        position_to_byte_offset(
            &snapshot,
            Position {
                line: 0,
                character: 2,
            },
        ),
        Ok(2)
    );
    assert_eq!(
        position_to_byte_offset(
            &snapshot,
            Position {
                line: 0,
                character: 3,
            },
        ),
        Ok(3)
    );
    assert_eq!(
        byte_offset_to_position(&snapshot, 4),
        Ok(Position {
            line: 0,
            character: 3,
        })
    );
    assert_eq!(
        position_to_byte_offset(
            &snapshot,
            Position {
                line: 1,
                character: 1,
            },
        ),
        Ok(7)
    );
    assert_eq!(
        position_to_byte_offset(
            &snapshot,
            Position {
                line: 1,
                character: 3,
            },
        ),
        Ok(11)
    );
    assert_eq!(
        position_to_byte_offset(
            &snapshot,
            Position {
                line: 1,
                character: 4,
            },
        ),
        Ok(12)
    );
    assert_eq!(
        position_to_byte_offset(
            &snapshot,
            Position {
                line: 2,
                character: 0,
            },
        ),
        Ok(13)
    );
    assert_eq!(
        byte_offset_to_position(&snapshot, 13),
        Ok(Position {
            line: 2,
            character: 0,
        })
    );
}

#[test]
fn positions_reject_surrogate_splits_and_out_of_range_values() {
    let snapshot = snapshot("abc\r\né𝄞z\n");

    assert_eq!(
        position_to_byte_offset(
            &snapshot,
            Position {
                line: 1,
                character: 2,
            },
        ),
        Err(PositionConversionError::InsideSurrogatePair {
            line: 1,
            character: 2,
        })
    );
    assert_eq!(
        position_to_byte_offset(
            &snapshot,
            Position {
                line: 1,
                character: 5,
            },
        ),
        Err(PositionConversionError::CharacterOutOfRange {
            line: 1,
            character: 5,
        })
    );
    assert_eq!(
        position_to_byte_offset(
            &snapshot,
            Position {
                line: 3,
                character: 0,
            },
        ),
        Err(PositionConversionError::LineOutOfRange { line: 3 })
    );
    assert_eq!(
        byte_offset_to_position(&snapshot, 6),
        Err(PositionConversionError::ByteOffsetOutOfRange { offset: 6 })
    );
    assert_eq!(
        byte_offset_to_position(&snapshot, 14),
        Err(PositionConversionError::ByteOffsetOutOfRange { offset: 14 })
    );
}

#[test]
fn file_uri_conversion_rejects_non_file_schemes() {
    let file_uri: Uri = "file:///phase4/example.fpas".parse().expect("file URI");
    let path = file_uri_to_path(&file_uri).expect("convert file URI");
    assert_eq!(
        path.file_name().and_then(|name| name.to_str()),
        Some("example.fpas")
    );

    let unsupported: Uri = "untitled:Functional-Pascal-1"
        .parse()
        .expect("untitled URI");
    let error = file_uri_to_path(&unsupported).expect_err("reject untitled URI");
    assert!(error.to_string().contains("unsupported URI scheme"));
}

#[test]
fn server_rejects_requests_before_initialize() {
    let transcript = run(&[shutdown(1), initialize(2), shutdown(3), exit()]);

    assert!(transcript.output.status.success());
    assert_eq!(
        response(&transcript.messages, 1)["error"]["code"],
        serde_json::json!(-32002)
    );
    assert!(response(&transcript.messages, 2).get("result").is_some());
    assert_eq!(
        response(&transcript.messages, 3)["result"],
        serde_json::Value::Null
    );
}

#[test]
fn server_rejects_a_non_string_standard_library_uri_and_continues() {
    let invalid_initialize = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": "file:///phase4",
            "capabilities": {},
            "initializationOptions": {"standardLibraryUri": 42}
        }
    });
    let transcript = run(&[invalid_initialize, initialize(2), shutdown(3), exit()]);

    assert!(transcript.output.status.success());
    assert_eq!(
        response(&transcript.messages, 1)["error"]["code"],
        serde_json::json!(-32602)
    );
    assert!(response(&transcript.messages, 2).get("result").is_some());
    assert_eq!(
        response(&transcript.messages, 3)["result"],
        serde_json::Value::Null
    );
}

#[test]
fn server_rejects_malformed_request_parameters_and_continues() {
    let malformed_shutdown = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "shutdown",
        "params": {"unexpected": true}
    });
    let transcript = run(&[
        initialize(1),
        initialized(),
        malformed_shutdown,
        shutdown(3),
        exit(),
    ]);

    assert!(transcript.output.status.success());
    assert_eq!(
        response(&transcript.messages, 2)["error"]["code"],
        serde_json::json!(-32602)
    );
    assert_eq!(
        response(&transcript.messages, 3)["result"],
        serde_json::Value::Null
    );
}

#[test]
fn initialized_registers_source_and_manifest_file_watchers() {
    let initialize = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": "file:///phase4",
            "capabilities": {
                "workspace": {
                    "didChangeWatchedFiles": {"dynamicRegistration": true}
                }
            }
        }
    });
    let transcript = run(&[initialize, initialized(), shutdown(2), exit()]);

    assert!(transcript.output.status.success());
    let registration = transcript
        .messages
        .iter()
        .find(|message| {
            message.get("method") == Some(&serde_json::json!("client/registerCapability"))
        })
        .expect("watched-file capability registration");
    assert_eq!(
        registration["params"]["registrations"][0]["method"],
        serde_json::json!("workspace/didChangeWatchedFiles")
    );
    assert_eq!(
        registration["params"]["registrations"][0]["registerOptions"]["watchers"],
        serde_json::json!([
            {"globPattern": "**/*.fpas"},
            {"globPattern": "**/*.fpasprj"},
            {"globPattern": "**/*.fpasworkspace"}
        ])
    );
}

#[test]
fn initialized_skips_file_watchers_for_clients_without_dynamic_registration() {
    let transcript = run(&[initialize(1), initialized(), shutdown(2), exit()]);

    assert!(transcript.output.status.success());
    assert!(transcript.messages.iter().all(|message| {
        message.get("method") != Some(&serde_json::json!("client/registerCapability"))
    }));
}
