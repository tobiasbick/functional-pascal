//! Named call contracts. See `docs/pascal/language/functions/parameters.md`.

use super::compile;

/// Starts a JSONL server at entry for call-expression integration tests.
/// See `docs/pascal/tools/debugger-jsonl.md`.
pub(super) fn server(source: &str) -> fpas_debug::jsonl::JsonlServer {
    let mut server = fpas_debug::jsonl::JsonlServer::new(fpas_debug::PreparedDebugTarget::new(
        compile(source),
        Vec::new(),
    ))
    .expect("named-call server");
    for request in [
        serde_json::json!({"type":"request","id":1,"command":"initialize","arguments":{"version":2}}),
        serde_json::json!({"type":"request","id":2,"command":"launch","arguments":{"stop_on_entry":true}}),
    ] {
        let _ = server.handle_line(&request.to_string());
    }
    server
}

static REQUEST_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(10);

/// Evaluates source text through the production parser and JSONL adapter.
/// See `docs/pascal/tools/debugger-jsonl.md`.
pub(super) fn evaluate(
    server: &mut fpas_debug::jsonl::JsonlServer,
    expression: &str,
) -> serde_json::Value {
    server.handle_line(&serde_json::json!({"type":"request","id":std::sync::atomic::AtomicU64::fetch_add(&REQUEST_ID, 1, std::sync::atomic::Ordering::Relaxed),"command":"evaluate",
        "arguments":{"expression":expression}}).to_string()).remove(0)
}

#[test]
fn routines_methods_and_enum_constructors_map_case_insensitive_names() {
    let mut server = server(
        "\
program NamedCalls;
type
  Pair = record
    Left: integer;
    static function Create(Left: integer): Pair;
    begin
      return Pair(Left := Left);
    end function;
    function Difference(Self: Pair; Right: integer; Scale: integer): integer;
    begin
      return (Self.Left - Right) * Scale;
    end function;
  end record;
type Choice = enum
    Both(Left: integer; Right: integer);
  end enum;
function Subtract(Left: integer; Right: integer): integer;
begin
  return Left - Right;
end function;
begin
end.",
    );
    for (expression, expected) in [
        ("Subtract(rIgHt := 2, LEFT := 9)", "7"),
        (
            "Pair.Create(Left := 9).Difference(Scale := 3, Right := 2)",
            "21",
        ),
        ("Choice.Both(Right := 2, Left := 9).Left", "9"),
        ("Choice.Both(Right := 2, Left := 9).Right", "2"),
    ] {
        let response = evaluate(&mut server, expression);
        assert_eq!(
            response["body"]["result"], expected,
            "{expression}: {response}"
        );
    }
}

#[test]
fn invalid_named_calls_fail_before_a_panicking_callee_executes() {
    let mut server = server(
        "\
program NamedErrors;
function Fail(Left: integer; Right: integer): integer;
begin
  return 1 div 0;
end function;
begin
end.",
    );
    for (expression, expected) in [
        (
            "Fail(Unknown := 1, Right := 2)",
            "no parameter or field `Unknown`",
        ),
        ("Fail(Left := 1)", "missing `right`"),
        ("Fail(Left := 1, left := 2)", "more than once"),
        ("Fail(Left := 1, 2)", "mix positional and named"),
    ] {
        let response = evaluate(&mut server, expression);
        assert_eq!(response["success"], false, "{response}");
        assert!(
            response
                .to_string()
                .to_ascii_lowercase()
                .contains(&expected.to_ascii_lowercase()),
            "{expression}: {response}"
        );
        assert!(
            !response.to_string().contains("division by zero"),
            "{response}"
        );
    }
}

#[test]
fn named_arguments_evaluate_once_in_written_order() {
    let mut server = server(
        "\
program NamedOrder;
var Counter: integer := 0;
function First(): integer;
begin
  Counter := 1;
  return Counter;
end function;
function Next(): integer;
begin
  Counter := Counter + 1;
  return Counter;
end function;
function Pack(Left: integer; Right: integer): integer;
begin
  return Left * 10 + Right;
end function;
begin
end.",
    );
    let response = evaluate(&mut server, "Pack(Right := First(), Left := Next())");
    assert_eq!(response["body"]["result"], "21", "{response}");
}
