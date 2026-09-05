use super::*;

#[test]
fn array_push_uses_direct_opcode_and_preserves_value_aliases() {
    let source = "\
program RegisterArrayPush;
uses Std.Array;
begin
  mutable var A: array of integer := [1];
  var Original: array of integer := A;
  Push(A, 2);
  if Length(Original) <> 1 then panic('array alias changed');
  if Length(A) <> 2 then panic('array push length mismatch');
  if A[1] <> 2 then panic('array push value mismatch')
end.";
    assert_succeeds(source);

    let program = super::parse_ok(source);
    let executable = crate::compile(&program).expect("compilation should succeed");
    assert!(
        executable
            .executable()
            .code
            .iter()
            .any(|instruction| { instruction.opcode() == Ok(fpas_bytecode::Opcode::ArrayPush) })
    );
}

#[test]
fn array_pop_uses_direct_opcode_and_preserves_value_aliases() {
    let source = r#"
program RegisterArrayPop;
uses Std.Array;
mutable var Global: array of integer := [4, 5];
begin
  mutable var A: array of integer := [1, 2];
  var Original: array of integer := A;
  if Pop(A) <> 2 then panic('last value');
  if Length(Original) <> 2 then panic('alias length');
  if Original[1] <> 2 then panic('alias value');
  if Pop(A) <> 1 then panic('first value');
  if Length(A) <> 0 then panic('empty length');
  if Pop(Global) <> 5 then panic('global value');
  if Length(Global) <> 1 then panic('global length');
  mutable var Captured: array of integer := [7, 8];
  var Take: function(): integer := function(): integer
  begin
    return Pop(Captured)
  end;
  if Take() <> 8 then panic('capture value');
  if Length(Captured) <> 1 then panic('capture length')
end."#;
    assert_succeeds(source);
    let executable = crate::compile(&super::super::parse_ok(source)).expect("compile");
    assert!(
        executable
            .executable()
            .code
            .iter()
            .any(|instruction| instruction.opcode() == Ok(fpas_bytecode::Opcode::ArrayPop))
    );
}
