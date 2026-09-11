use super::*;

#[test]
fn local_dictionary_writes_preserve_aliases_and_insert_keys() {
    assert_succeeds(
        r#"
program LocalDictionary;
begin
  mutable var Values: dict of string to integer := ['a': 1];
  var Original: dict of string to integer := Values;
  Values['a'] := 9;
  Values['b'] := Values['a'] + 1;
  if Original['a'] <> 1 then panic('alias changed');
  if 'b' in Original then panic('alias gained key');
  if Values['b'] <> 10 then panic('insert failed')
end.
"#,
    );
}

#[test]
fn local_index_fallback_preserves_captured_root_snapshot_and_rhs_order() {
    assert_succeeds(
        r#"
program LocalIndexOrder;
procedure Check();
begin
  mutable var Values: array of integer := [1, 2];
  mutable var Order: integer := 0;
  var Index: function(): integer := function(): integer
  begin
    Order := Order * 10 + 2;
    Values := [7, 8];
    return 0
  end;
  var Replacement: function(): integer := function(): integer
  begin
    Order := Order * 10 + 1;
    return 9
  end;
  Values[Index()] := Replacement();
  if Order <> 12 then panic('evaluation order');
  if (Values[0] <> 9) or (Values[1] <> 2) then panic('root snapshot');
  Values[1] := 4;
  if Values[1] <> 4 then panic('captured direct index')
end;
begin
  Check()
end.
"#,
    );
}

#[test]
fn array_push_uses_direct_opcode_and_preserves_value_aliases() {
    let source = "\
program RegisterArrayPush;
uses Std.Arrays;
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
uses Std.Arrays;
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

#[test]
fn local_index_write_mutates_its_local_register_without_a_collection_move() {
    let source = r#"
program RegisterLocalIndexWrite;
begin
  mutable var Values: array of integer := [1, 2];
  var Original: array of integer := Values;
  Values[1] := 9;
  if Original[1] <> 2 then panic('array alias changed');
  if Values[1] <> 9 then panic('array value mismatch')
end.
"#;
    assert_succeeds(source);

    let ast = super::super::parse_ok(source);
    let ir = crate::lower(&ast).expect("lowering should succeed");
    assert!(
        ir.functions
            .iter()
            .flat_map(|function| &function.blocks)
            .flat_map(|block| &block.instructions)
            .any(|instruction| {
                matches!(
                    instruction.operation,
                    fpas_ir::Operation::StoreLocalIndex { .. }
                )
            })
    );

    let executable = crate::compile(&ast).expect("compilation should succeed");
    let code = &executable.executable().code;
    let index_set = code
        .iter()
        .position(|instruction| instruction.opcode() == Ok(fpas_bytecode::Opcode::IndexSet))
        .expect("local index write opcode");
    assert!(
        index_set == 0
            || code[index_set - 1].opcode() != Ok(fpas_bytecode::Opcode::Move)
            || code[index_set - 1].abc_payload().a != code[index_set].abc_payload().a,
        "local index write must not copy the collection before mutation"
    );
}
