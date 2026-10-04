//! Computed initialization and inferred storage through compiler/VM execution.

use super::assert_succeeds;

#[test]
fn computed_bindings_run_once_in_declaration_order_and_keep_snapshots() {
    assert_succeeds("program T; var Calls: integer := 0;
        function Next(): integer; begin Calls := Calls + 1; return Calls; end function;
        const First: integer := Next(); const Second: integer := Next();
        begin const Third := Next(); var Copy := Third; Copy := Copy + 1;
          if (First <> 1) or (Second <> 2) or (Third <> 3) or (Copy <> 4) or (Calls <> 3) then panic('binding evaluation order'); end if;
        end program;");
}

#[test]
fn inferred_collections_and_records_isolate_nested_mutable_storage() {
    assert_succeeds("program T; type Box of (T) = record Values: array of (T); end record;
        function Edit(Value: Box of (integer)): Box of (integer);
        begin var Copy := Value; Copy.Values[0] := 9; return Copy; end function;
        begin const Original := Box(Values := [1, 2]); var Changed := Edit(Original);
          var Mapping := ['one': [Original]]; const Snapshot := Mapping;
          Mapping['one'][0].Values[1] := 7;
          if (Original.Values[0] <> 1) or (Changed.Values[0] <> 9) or (Snapshot['one'][0].Values[1] <> 2) or (Mapping['one'][0].Values[1] <> 7) then panic('inferred value copy'); end if;
        end program;");
}

#[test]
fn inferred_named_captures_and_stateful_closure_copies_keep_their_ownership() {
    assert_succeeds("program T; function Make(Base: integer): function(): integer;
        function GetValue(): integer; begin return Offset; end function;
        function Compute(): integer; begin return Base + 1; end function;
        begin const Offset := Compute(); return GetValue; end function;
        begin const GetValue := Make(41); if GetValue() <> 42 then panic('inferred nested capture'); end if;
          var Count := 0; const Increment := function(): integer begin Count := Count + 1; return Count; end function;
          const Copy := Increment;
          if (Increment() <> 1) or (Copy() <> 2) or (Count <> 2) then panic('closure identity'); end if;
        end program;");
}
