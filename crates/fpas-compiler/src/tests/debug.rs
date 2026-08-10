use fpas_bytecode::DebugBindingKind;

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
    assert!(
        image
            .functions
            .iter()
            .any(|function| { function.debug.bindings.iter().any(|binding| binding.hidden) })
    );
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
