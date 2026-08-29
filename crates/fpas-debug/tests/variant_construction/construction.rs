//! Successful JSONL variant construction and continuation contracts.

use super::*;

#[test]
fn jsonl_variant_construct_commits_and_continues() {
    let mut server = server();
    let mut id = initialize(&mut server);
    let current = stop_with_initialized_locals(&mut server, &mut id);
    let empty = send(
        &mut server,
        &mut id,
        "variant.construct",
        json!({
            "frame_id": current,
            "target": "Selected",
            "variant": "Choice.Empty",
            "fields": {}
        }),
    );
    assert_eq!(empty[0]["success"], true, "{empty:?}");
    assert_eq!(empty[0]["body"]["variant"], "Choice.Empty");
    assert_eq!(empty[0]["body"]["result"], "Choice.Empty");

    let current = frame(&mut server, &mut id);
    let pair = send(
        &mut server,
        &mut id,
        "variant.construct",
        json!({
            "frame_id": current,
            "target": "Selected",
            "variant": "choice.pair",
            "fields": {"Right": "Next()", "Left": "Next()"}
        }),
    );
    assert_eq!(pair[0]["success"], true, "{pair:?}");
    assert_eq!(pair[0]["body"]["variant"], "Choice.Pair");

    let current = frame(&mut server, &mut id);
    let locals = locals_reference(&mut server, &mut id, current);
    let selected = named_variable(&mut server, &mut id, locals, "Selected");
    assert_eq!(selected["value"], "Choice.Pair");
    let fields = named_variable(
        &mut server,
        &mut id,
        selected["variables_reference"]
            .as_u64()
            .expect("pair fields"),
        "Left",
    );
    let right = named_variable(
        &mut server,
        &mut id,
        selected["variables_reference"]
            .as_u64()
            .expect("pair fields"),
        "Right",
    );
    assert_eq!(fields["value"], "1");
    assert_eq!(right["value"], "2");

    construct(
        &mut server,
        &mut id,
        "PackedHolder.Item",
        "PackedHolder.Item",
        "Choice.Count",
        json!({"Value": "4"}),
        "Choice.Count",
    );
    construct(
        &mut server,
        &mut id,
        "Failed",
        "Failed",
        "Ok",
        json!({"value": "9"}),
        "Ok(...)",
    );
    construct(
        &mut server,
        &mut id,
        "Scores['blue']",
        "Scores['blue']",
        "Choice.Count",
        json!({"Value": "5"}),
        "Choice.Count",
    );
    construct(
        &mut server,
        &mut id,
        "WrappedChoice.value",
        "WrappedChoice",
        "Choice.Pair",
        json!({"Left": "6", "Right": "7"}),
        "Ok(...)",
    );
    construct(
        &mut server,
        &mut id,
        "WrappedChoices[0].value",
        "WrappedChoices[0]",
        "Choice.Count",
        json!({"Value": "8"}),
        "Ok(...)",
    );
    construct(
        &mut server,
        &mut id,
        "OuterValue.Item",
        "OuterValue",
        "Choice.Count",
        json!({"Value": "9"}),
        "Outer.First",
    );

    let _ = send(&mut server, &mut id, "continue", json!({}));
    let records = server.wait();
    let output = records
        .iter()
        .filter(|record| record["event"] == "output")
        .map(|record| record["body"]["text"].as_str().unwrap_or_default())
        .collect::<String>();
    assert_eq!(output, "3\n4\n0\n5\n0\n9\n13\n8\n9\n99\n");
}

fn construct(
    server: &mut JsonlServer,
    id: &mut u64,
    target: &str,
    observed_expression: &str,
    variant: &str,
    fields: Value,
    expected: &str,
) {
    let current = frame(server, id);
    let result = send(
        server,
        id,
        "variant.construct",
        json!({
            "frame_id": current,
            "target": target,
            "variant": variant,
            "fields": fields
        }),
    );
    assert_eq!(result[0]["success"], true, "{target}: {result:?}");
    let current = frame(server, id);
    let observed = send(
        server,
        id,
        "evaluate",
        json!({"frame_id": current, "expression": observed_expression}),
    );
    assert_eq!(
        observed[0]["body"]["result"], expected,
        "{target}: {observed:?}"
    );
}
