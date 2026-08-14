use fpas_bytecode::{DebugBindingKind, DebugType};

use super::parse_ok;

#[test]
fn compiler_retains_source_bindings_scopes_and_sequence_points() {
    let program = parse_ok(
        r#"
program DebugMetadata;

function Add(Value: integer): integer;
begin
  var Offset: integer := 1;
  begin
    var Nested: integer := Value + Offset;
    if Nested < 0 then
      panic('unreachable')
  end;
  return Value + Offset
end;

begin
  var Answer: integer := Add(41);
  if Answer <> 42 then
    panic('wrong answer')
end.
"#,
    );
    let executable = crate::compile(&program).expect("debug metadata source should compile");
    let image = executable.executable();
    let add = image
        .functions
        .iter()
        .find(|function| image.strings.get(function.name) == Some("add"))
        .expect("Add function metadata");

    let binding = |name: &str| {
        add.debug
            .bindings
            .iter()
            .find(|binding| image.strings.get(binding.name) == Some(name))
            .expect("named debug binding")
    };
    assert_eq!(binding("Value").kind, DebugBindingKind::Parameter);
    assert_eq!(binding("Offset").kind, DebugBindingKind::Local);
    assert_eq!(binding("Nested").kind, DebugBindingKind::Local);
    assert_eq!(
        image.debug_types.get(binding("Value").ty.get() as usize),
        Some(&DebugType::Integer)
    );
    assert_eq!(
        add.debug
            .result_type
            .and_then(|ty| image.debug_types.get(ty.get() as usize)),
        Some(&DebugType::Integer),
        "source functions retain a portable result type"
    );
    assert_ne!(binding("Nested").scope, binding("Offset").scope);
    assert!(
        !add.debug.sequence_points.is_empty(),
        "source-bearing instructions should produce debugger sequence points"
    );
    assert!(
        add.debug
            .sequence_points
            .windows(2)
            .all(|pair| pair[0].instruction < pair[1].instruction),
        "sequence points must follow executable instruction order"
    );
}

#[test]
fn compiler_marks_capture_cells_and_hidden_loop_storage() {
    let program = parse_ok(
        r#"
program DebugCaptureMetadata;

function Counter(): function(): integer;
begin
  mutable var Value: integer := 0;
  return function(): integer begin
    Value := Value + 1;
    return Value
  end
end;

begin
  var Next: function(): integer := Counter();
  for Index: integer := 1 to 2 do
    Next()
end.
"#,
    );
    let executable = crate::compile(&program).expect("capture metadata source should compile");
    let image = executable.executable();
    let capture = image
        .functions
        .iter()
        .flat_map(|function| &function.debug.bindings)
        .find(|binding| {
            binding.kind == DebugBindingKind::Capture
                && image.strings.get(binding.name) == Some("Value")
        })
        .expect("captured Value binding");
    assert!(capture.cell_backed);
    assert_eq!(
        image.debug_types.get(capture.ty.get() as usize),
        Some(&DebugType::Integer),
        "mutable captures expose their assignable inner type"
    );
    assert!(
        image
            .functions
            .iter()
            .any(|function| { function.debug.bindings.iter().any(|binding| binding.hidden) })
    );
}

#[test]
fn compiler_retains_structured_debug_types_for_roots_and_aggregate_children() {
    let program = parse_ok(
        r#"
program DebugStructuredTypes;

type
  Box = record
    Value: integer;
  end;

mutable var
  Scores: dict of string to integer := ['Ada': 1];

begin
  mutable var Item: Box := record
    Value := 2;
  end;
  mutable var Items: array of integer := [3];
  var Maybe: option of integer := Some(4)
end.
"#,
    );
    let executable = crate::compile(&program).expect("structured debug types should compile");
    let image = executable.executable();
    assert!(
        image
            .debug_types
            .iter()
            .any(|ty| matches!(ty, DebugType::Dictionary { .. }))
    );
    assert!(
        image
            .debug_types
            .iter()
            .any(|ty| matches!(ty, DebugType::Array(_)))
    );
    assert!(
        image
            .debug_types
            .iter()
            .any(|ty| matches!(ty, DebugType::Option(_)))
    );
    let record = &image.records[0];
    assert_eq!(
        image.debug_types.get(record.fields[0].ty.get() as usize),
        Some(&DebugType::Integer)
    );
    let global = image
        .globals
        .iter()
        .find(|global| image.strings.get(global.name) == Some("Scores"))
        .expect("global metadata");
    assert!(matches!(
        image.debug_types.get(global.ty.get() as usize),
        Some(DebugType::Dictionary { .. })
    ));
}

#[test]
fn compiler_retains_shadowed_bindings_and_same_line_sequence_columns() {
    let program = parse_ok(
        r#"
program DebugShadowMetadata;

begin
  var Value: integer := 1;
  begin
    var Value: integer := 2; var Other: integer := Value + 1;
    if Other <> 3 then panic('wrong inner value')
  end;
  if Value <> 1 then panic('wrong outer value')
end.
"#,
    );
    let executable = crate::compile(&program).expect("shadow metadata source should compile");
    let image = executable.executable();
    let root = &image.functions[0];
    let shadowed = root
        .debug
        .bindings
        .iter()
        .filter(|binding| image.strings.get(binding.name) == Some("Value"))
        .collect::<Vec<_>>();
    assert_eq!(shadowed.len(), 2);
    assert_ne!(shadowed[0].scope, shadowed[1].scope);
    assert!(
        root.debug
            .sequence_points
            .iter()
            .enumerate()
            .any(|(index, left)| {
                root.debug.sequence_points[index + 1..].iter().any(|right| {
                    left.location.line == right.location.line
                        && left.location.column != right.location.column
                })
            })
    );
}

#[test]
fn compiler_retains_distinct_task_result_types_for_local_and_global_bindings() {
    let program = parse_ok(
        r#"
program DebugTaskResultTypes;

uses Std.Task;

function Seven(): integer;
begin
  return 7
end;

function Label(): string;
begin
  return 'nope'
end;

var GlobalCurrent: task := go Seven();
var GlobalWrong: task := go Label();

begin
  var Current: task := go Seven();
  var Wrong: task := go Label()
end.
"#,
    );
    let executable = crate::compile(&program).expect("task debug types should compile");
    let image = executable.executable();
    let binding_type = |name: &str| {
        let binding = image.functions[0]
            .debug
            .bindings
            .iter()
            .find(|binding| image.strings.get(binding.name) == Some(name))
            .expect("named task binding");
        image
            .debug_types
            .get(binding.ty.get() as usize)
            .expect("task debug type")
    };
    let global_type = |name: &str| {
        let binding = image
            .globals
            .iter()
            .find(|binding| image.strings.get(binding.name) == Some(name))
            .expect("named global task binding");
        image
            .debug_types
            .get(binding.ty.get() as usize)
            .expect("global task debug type")
    };
    for (current, wrong) in [
        (binding_type("Current"), binding_type("Wrong")),
        (global_type("GlobalCurrent"), global_type("GlobalWrong")),
    ] {
        let (DebugType::Task(current_result), DebugType::Task(wrong_result)) = (current, wrong)
        else {
            panic!("expected task bindings, got {current:?} and {wrong:?}");
        };
        assert_ne!(current_result, wrong_result);
        assert_eq!(
            image.debug_types.get(current_result.get() as usize),
            Some(&DebugType::Integer)
        );
        assert_eq!(
            image.debug_types.get(wrong_result.get() as usize),
            Some(&DebugType::String)
        );
    }
}
