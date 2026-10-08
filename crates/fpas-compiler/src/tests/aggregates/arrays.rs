use super::*;

#[test]
fn local_dictionary_writes_preserve_aliases_and_insert_keys() {
    assert_succeeds(
        r#"
program LocalDictionary;
begin
  var Values: dict of string to integer := ['a': 1];
  const Original: dict of string to integer := Values;
  Values['a'] := 9;
  Values['b'] := Values['a'] + 1;
  if Original['a'] <> 1 then panic('alias changed'); end if;
  if 'b' in Original then panic('alias gained key'); end if;
  if Values['b'] <> 10 then panic('insert failed'); end if;
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
  var Values: array of integer := [1, 2];
  var Order: integer := 0;
  const Index: function(): integer := function(): integer
  begin
    Order := Order * 10 + 2;
    Values := [7, 8];
    return 0;
  end function;
  const Replacement: function(): integer := function(): integer
  begin
    Order := Order * 10 + 1;
    return 9;
  end function;
  Values[Index()] := Replacement();
  if Order <> 12 then panic('evaluation order'); end if;
  if (Values[0] <> 9) or (Values[1] <> 2) then panic('root snapshot'); end if;
  Values[1] := 4;
  if Values[1] <> 4 then panic('captured direct index'); end if;
end procedure;
begin
  Check();
end.
"#,
    );
}

#[test]
fn array_push_uses_direct_opcode_and_preserves_value_aliases() {
    let source = "program RegisterArrayPush;\n\nbegin\n  var A: array of integer := [1];\n  const Original: array of integer := A;\n  A.Push(2);\n  if Original.Length() <> 1 then panic('array alias changed'); end if;\n  if A.Length() <> 2 then panic('array push length mismatch'); end if;\n  if A[1] <> 2 then panic('array push value mismatch'); end if;\nend.";
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

var Global: array of integer := [4, 5];
begin
  var A: array of integer := [1, 2];
  const Original: array of integer := A;
  if A.Pop() <> 2 then panic('last value'); end if;
  if Original.Length() <> 2 then panic('alias length'); end if;
  if Original[1] <> 2 then panic('alias value'); end if;
  if A.Pop() <> 1 then panic('first value'); end if;
  if A.Length() <> 0 then panic('empty length'); end if;
  if Global.Pop() <> 5 then panic('global value'); end if;
  if Global.Length() <> 1 then panic('global length'); end if;
  var Captured: array of integer := [7, 8];
  const Take: function(): integer := function(): integer
  begin
    return Captured.Pop();
  end function;
  if Take() <> 8 then panic('capture value'); end if;
  if Captured.Length() <> 1 then panic('capture length'); end if;
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
  var Values: array of integer := [1, 2];
  const Original: array of integer := Values;
  Values[1] := 9;
  if Original[1] <> 2 then panic('array alias changed'); end if;
  if Values[1] <> 9 then panic('array value mismatch'); end if;
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
