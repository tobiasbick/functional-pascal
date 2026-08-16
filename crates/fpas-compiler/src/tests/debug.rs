#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "compiler debug tests keep fixture failures local"
)]

use fpas_bytecode::{DebugBindingKind, DebugType, Opcode};

use super::parse_ok;

#[test]
fn compiler_retains_exact_record_method_mappings_for_debugger_binding() {
    let program = parse_ok(
        r#"
program DebugBoundMethod;
type Counter = record
  Base: integer;
  function Add(Self: Counter; Value: integer): integer;
  begin
    return Self.Base + Value
  end;
end;
begin
  var C: Counter := record Base := 2; end;
  if C.Add(3) <> 5 then panic('wrong')
end.
"#,
    );
    let executable = crate::compile(&program).expect("record method source should compile");
    let image = executable.executable();
    let record = image.records.first().expect("Counter layout");
    let method = record.methods.first().expect("Counter.Add mapping");

    assert_eq!(image.strings.get(record.name), Some("Counter"));
    assert_eq!(image.strings.get(method.name), Some("Add"));
    assert_eq!(image.strings.get(method.routine), Some("Counter.Add"));
}

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
    for name in ["Offset", "Nested"] {
        let binding = binding(name);
        let initializer = binding.initializer.expect("local initializer store");
        let instruction = image.code[initializer.get() as usize];
        assert_eq!(instruction.opcode(), Ok(Opcode::Move));
        assert_eq!(instruction.abc_payload().a, binding.register.get());
    }
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
    let initializer = global.initializer.expect("global initializer store");
    let instruction = image.code[initializer.instruction.get() as usize];
    assert_eq!(initializer.function, image.entry);
    assert_eq!(instruction.opcode(), Ok(Opcode::StoreGlobal));
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

#[test]
fn compiler_records_exact_capture_provenance_for_named_nested_routines() {
    let program = parse_ok(
        r#"
program CaptureProvenance;

type
  Handler = function(Value: integer): integer;

function MakeAdder(Base: integer): Handler;
  function AddBase(Value: integer): integer;
  begin
    return Base + Value
  end;
begin
  return AddBase
end;

function Outer(Offset: integer): integer;
  function AddOffset(Value: integer): integer;
  begin
    return Value + Offset
  end;
begin
  begin
    var Offset: integer := 99;
    return AddOffset(1)
  end
end;

function Mutating(): Handler;
  function AddCell(Value: integer): integer;
  begin
    Cell := Cell + 1;
    return Value + Cell
  end;
begin
  mutable var Cell: integer := 1;
  return AddCell
end;

function OuterCell(): Handler;
  function Mid(): Handler;
    function AddEnclosed(Value: integer): integer;
    begin
      Cell := Cell + 1;
      return Value + Cell
    end;
  begin
    var Keep: integer := Cell;
    return AddEnclosed
  end;
begin
  mutable var Cell: integer := 1;
  return Mid()
end;

begin
  var First: Handler := MakeAdder(10);
  var Answer: integer := Outer(7);
  var Next: Handler := Mutating();
  var Enclosed: Handler := OuterCell()
end.
"#,
    );
    let executable = crate::compile(&program).expect("capture provenance should compile");
    let image = executable.executable();
    let function = |name: &str| {
        image
            .functions
            .iter()
            .find(|function| image.strings.get(function.name) == Some(name))
            .expect("named function")
    };
    let add_base = function("makeadder.addbase");
    assert_eq!(add_base.capture_count, 1);
    let owner = add_base.debug.lexical_owner.expect("lexical owner");
    assert_eq!(
        image
            .strings
            .get(image.functions[usize::from(owner.get())].name),
        Some("makeadder")
    );
    assert_eq!(add_base.debug.capture_sources.len(), 1);
    assert_eq!(
        add_base.debug.capture_sources[0].kind,
        fpas_bytecode::DebugCaptureKind::Value
    );
    let owner_binding = &image.functions[usize::from(owner.get())].debug.bindings
        [add_base.debug.capture_sources[0].binding.get() as usize];
    assert_eq!(image.strings.get(owner_binding.name), Some("Base"));
    assert!(!owner_binding.cell_backed);

    let add_offset = function("outer.addoffset");
    let outer = add_offset.debug.lexical_owner.expect("outer owner");
    let outer_function = &image.functions[usize::from(outer.get())];
    let captured =
        &outer_function.debug.bindings[add_offset.debug.capture_sources[0].binding.get() as usize];
    assert_eq!(image.strings.get(captured.name), Some("Offset"));
    assert_eq!(captured.kind, DebugBindingKind::Parameter);
    let shadow_count = outer_function
        .debug
        .bindings
        .iter()
        .filter(|binding| image.strings.get(binding.name) == Some("Offset"))
        .count();
    assert_eq!(shadow_count, 2);

    let add_cell = function("mutating.addcell");
    assert_eq!(
        add_cell.debug.capture_sources[0].kind,
        fpas_bytecode::DebugCaptureKind::Cell
    );

    let add_enclosed = function("outercell.mid.addenclosed");
    assert_eq!(
        add_enclosed.debug.capture_sources[0].kind,
        fpas_bytecode::DebugCaptureKind::EnclosingCell
    );
    let mid = function("outercell.mid");
    assert_eq!(
        mid.debug.capture_sources[0].kind,
        fpas_bytecode::DebugCaptureKind::Cell
    );
}

#[test]
fn same_named_nested_routines_keep_distinct_capture_identity() {
    let program = parse_ok(
        r#"
program DistinctNestedCaptures;

type
  Handler = function(Value: integer): integer;

function FactoryA(A: integer): Handler;
  function Apply(Value: integer): integer;
  begin
    return A + Value
  end;
begin
  return Apply
end;

function FactoryB(B: integer): Handler;
  function Apply(Value: integer): integer;
  begin
    return B + Value
  end;
begin
  return Apply
end;

begin
  var First: Handler := FactoryA(1);
  var Second: Handler := FactoryB(2)
end.
"#,
    );
    let executable = crate::compile(&program).expect("same-named nested routines should compile");
    let image = executable.executable();

    for (routine_name, source_name) in [("factorya.apply", "A"), ("factoryb.apply", "B")] {
        let routine = image
            .functions
            .iter()
            .find(|function| image.strings.get(function.name) == Some(routine_name))
            .expect("nested routine");
        let owner = routine.debug.lexical_owner.expect("lexical owner");
        let source = routine.debug.capture_sources[0];
        let binding = &image.functions[usize::from(owner.get())].debug.bindings
            [source.binding.get() as usize];
        assert_eq!(image.strings.get(binding.name), Some(source_name));
    }
}
