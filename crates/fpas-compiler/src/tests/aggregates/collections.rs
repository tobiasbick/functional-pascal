use super::*;

#[test]
fn globals_arrays_and_dictionaries_execute() {
    assert_succeeds(
        r#"program RegisterCollections;
  mutable var Total: integer := 1;
begin
  mutable var Values: array of (integer) := [2, 3, 4];
  Values[1] := 8;
  mutable var Lookup: dict of (string, integer) := ['a': 5];
  Lookup['b'] := 7;
  Total := Total + Values[1] + Lookup['b'];
  if (Total <> 16) or not (8 in Values) or not ('b' in Lookup) then
    panic('collection mismatch'); end if;
end program;"#,
    );
}

#[test]
fn string_indexing_and_membership_execute() {
    assert_succeeds(
        r#"program RegisterStringAggregateOps;
begin
  var Text: string := 'Hällo';
  if Text[1] <> 'ä' then panic('unicode string index mismatch'); end if;
  if not ('äll' in Text) then panic('substring membership mismatch'); end if;
  if not ('ä' in Text) then panic('character membership mismatch'); end if;
end program;"#,
    );
}

#[test]
fn global_nested_index_write_uses_direct_path_and_preserves_value_aliases() {
    let source = r#"program RegisterGlobalIndexPath;
  mutable var Surface: array of (array of (integer)) := [[1, 2]];
begin
  var Original: array of (array of (integer)) := Surface;
  Surface[0][1] := 9;
  if Original[0][1] <> 2 then panic('global alias changed'); end if;
  if Surface[0][1] <> 9 then panic('global path value mismatch'); end if;
end program;"#;
    assert_succeeds(source);

    let program = super::parse_ok(source);
    let executable = crate::compile(&program).expect("compilation should succeed");
    assert!(executable.executable().code.iter().any(|instruction| {
        instruction.opcode() == Ok(fpas_bytecode::Opcode::StoreGlobalIndexPath)
    }));
}

#[test]
fn global_nested_index_write_preserves_index_side_effect_order() {
    assert_succeeds(
        r#"program RegisterGlobalIndexOrder;
  mutable var Surface: array of (array of (integer)) := [[1, 2]];
function ChangeSurface(): integer;
begin
  Surface := [[3, 4]];
  return 1;
end function;
begin
  Surface[0][ChangeSurface()] := 9;
  if Surface[0][0] <> 1 then panic('snapshot order changed'); end if;
  if Surface[0][1] <> 9 then panic('snapshot update missing'); end if;
end program;"#,
    );
}

#[test]
fn global_nested_dictionary_write_inserts_leaf_and_preserves_aliases() {
    assert_succeeds(
        r#"program RegisterGlobalDictionaryPath;
  mutable var Lookup: dict of (string, dict of (string, integer)) := ['outer': ['old': 1]];
begin
  var Original: dict of (string, dict of (string, integer)) := Lookup;
  Lookup['outer']['new'] := 2;
  if 'new' in Original['outer'] then panic('dictionary alias changed'); end if;
  if Lookup['outer']['new'] <> 2 then panic('dictionary path value mismatch'); end if;
end program;"#,
    );
}
