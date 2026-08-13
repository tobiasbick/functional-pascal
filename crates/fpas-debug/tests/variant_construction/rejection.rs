//! Atomic JSONL rejection and stopped-state contracts.

use super::*;

#[test]
fn jsonl_variant_construct_rejects_without_mutation() {
    let mut server = server();
    let mut id = initialize(&mut server);
    let current = stop_with_initialized_locals(&mut server, &mut id);
    let parse = send(
        &mut server,
        &mut id,
        "variant.construct",
        json!({
            "frame_id": current,
            "target": "Selected",
            "variant": "Choice.Count",
            "fields": {"Value": "1 +"}
        }),
    );
    assert_eq!(parse[0]["success"], false);
    assert!(parse[0]["error"]["offset"].is_number(), "{parse:?}");
    assert!(
        !parse[0]["error"]["help"]
            .as_str()
            .unwrap_or_default()
            .is_empty()
    );

    let unknown = send(
        &mut server,
        &mut id,
        "variant.construct",
        json!({
            "frame_id": current,
            "target": "Selected",
            "variant": "Choice.Nope",
            "fields": {}
        }),
    );
    assert_eq!(unknown[0]["error"]["code"], "variant_unknown");

    let missing = send(
        &mut server,
        &mut id,
        "variant.construct",
        json!({
            "frame_id": current,
            "target": "Selected",
            "variant": "Choice.Pair",
            "fields": {"Left": "1"}
        }),
    );
    assert_eq!(missing[0]["error"]["code"], "variant_field_set");

    let extra = send(
        &mut server,
        &mut id,
        "variant.construct",
        json!({
            "frame_id": current,
            "target": "Selected",
            "variant": "Choice.Empty",
            "fields": {"Value": "1"}
        }),
    );
    assert_eq!(extra[0]["error"]["code"], "variant_field_set");

    let duplicate = send(
        &mut server,
        &mut id,
        "variant.construct",
        json!({
            "frame_id": current,
            "target": "Selected",
            "variant": "Choice.Pair",
            "fields": {"Left": "1", "left": "2", "Right": "3"}
        }),
    );
    assert_eq!(duplicate[0]["error"]["code"], "variant_field_set");

    let omitted = send(
        &mut server,
        &mut id,
        "variant.construct",
        json!({
            "frame_id": current,
            "target": "Selected",
            "variant": "Choice.Empty"
        }),
    );
    assert_eq!(omitted[0]["error"]["code"], "invalid_request");

    let check = send(
        &mut server,
        &mut id,
        "evaluate",
        json!({"frame_id": current, "expression": "Selected"}),
    );
    assert_eq!(check[0]["success"], true, "{check:?}");
    assert_eq!(check[0]["body"]["result"], "Choice.Empty");

    let running = send(&mut server, &mut id, "continue", json!({}));
    assert_eq!(running[0]["success"], true);
    let rejected = send(
        &mut server,
        &mut id,
        "variant.describe",
        json!({"target": "Selected"}),
    );
    assert_eq!(rejected[0]["error"]["code"], "invalid_state");
}
