//! Read-only JSONL variant discovery contracts.

use super::*;

#[test]
fn jsonl_variant_describe_is_read_only_and_canonical() {
    let mut server = server();
    let mut id = initialize(&mut server);
    let current = stop_with_initialized_locals(&mut server, &mut id);
    let described = send(
        &mut server,
        &mut id,
        "variant.describe",
        json!({"frame_id": current, "target": "Selected"}),
    );
    assert_eq!(described[0]["success"], true, "{described:?}");
    assert_eq!(described[0]["body"]["target"], "Selected");
    assert_eq!(described[0]["body"]["type_name"], "Choice");
    let names = described[0]["body"]["variants"]
        .as_array()
        .expect("variants")
        .iter()
        .map(|variant| variant["name"].as_str().expect("name"))
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "Choice.Empty",
            "Choice.Count",
            "Choice.Pair",
            "Choice.Nested"
        ]
    );
    assert!(
        described[0]["body"]["variants"][0]["fields"]
            .as_array()
            .expect("empty")
            .is_empty()
    );
    assert_eq!(
        described[0]["body"]["variants"][2]["fields"][0]["name"],
        "Left"
    );
    assert_eq!(
        described[0]["body"]["variants"][2]["fields"][0]["type_name"],
        "Integer"
    );

    let holder = send(
        &mut server,
        &mut id,
        "variant.describe",
        json!({"frame_id": current, "target": "PackedHolder"}),
    );
    assert_eq!(holder[0]["success"], false);
    assert_eq!(holder[0]["error"]["code"], "variable_path_unsupported");

    let outcome = send(
        &mut server,
        &mut id,
        "variant.describe",
        json!({"frame_id": current, "target": "Outcome"}),
    );
    assert_eq!(outcome[0]["body"]["variants"][0]["name"], "Ok");
    assert_eq!(outcome[0]["body"]["variants"][1]["name"], "Error");

    let optional = send(
        &mut server,
        &mut id,
        "variant.describe",
        json!({"frame_id": current, "target": "Missing"}),
    );
    assert_eq!(optional[0]["body"]["variants"][1]["name"], "None");
    assert!(
        optional[0]["body"]["variants"][1]["fields"]
            .as_array()
            .expect("none")
            .is_empty()
    );
}
