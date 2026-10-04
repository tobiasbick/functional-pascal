//! Ordinary source calls preserve explicit caller storage and value snapshots.

use crate::tests::{assert_succeeds, run_program};

#[test]
fn source_var_calls_mutate_locals_globals_fields_and_frozen_elements() {
    assert_succeeds(
        r#"program References;
type Box = record Item: array of (integer); end record;
 var Global: integer := 1;
procedure Increase(var Value: integer); begin Value := Value + 1; end procedure;
procedure Forward(var Value: integer); begin Increase(var Value); Value := Value + 10; end procedure;
begin
   var Local: integer := 1;
  const Snapshot: integer := Local;
  Forward(var Local);
  Increase(var Global);
   var Data: Box := Box(Item := [4]);
  const Old: Box := Data;
  Increase(var Data.Item[0]);
   var Mapping: dict of (string, integer) := ['key': 8];
  Increase(var Mapping['key']);
  if (Local <> 12) or (Snapshot <> 1) or (Global <> 2) or (Data.Item[0] <> 5) or (Old.Item[0] <> 4) or (Mapping['key'] <> 9) then panic('reference or snapshot mismatch'); end if;
end program;"#,
    );
}

#[test]
fn source_var_modes_survive_indirect_returned_indexed_and_field_calls() {
    assert_succeeds(
        r#"program Targets;
type Action = procedure(var Value: integer);
type Holder = record Action: Action; end record;
procedure Increase(var Value: integer); begin Value := Value + 1; end procedure;
function Select(): Action; begin return Increase; end function;
begin
   var Count: integer := 0;
  const F: Action := Increase;
  const Functions: array of (Action) := [F];
  const Data: Holder := Holder(Action := F);
  F(var Count);
  Select()(var Count);
  Functions[0](var Count);
  Data.Action(var Count);
  (F)(var Count);
  if Count <> 5 then panic('indirect modes'); end if;
end program;"#,
    );
}

#[test]
fn source_generic_var_function_and_anonymous_action_use_reference_storage() {
    assert_succeeds(
        r#"program GenericReferences;
function Replace of (T)(var Value: T; NewValue: T): T;
begin const Previous: T := Value; Value := NewValue; return Previous; end function;
begin
   var Number: integer := 1;
  const Before: integer := Replace(var Number, 42);
   var Text: string := 'old';
  const Old: string := Replace(var Text, 'new');
  const Act: procedure(var Value: integer) := procedure(var Value: integer) begin Value := Value + 1; end procedure;
  Act(var Number);
  if (Before <> 1) or (Number <> 43) or (Old <> 'old') or (Text <> 'new') then panic('generic modes'); end if;
end program;"#,
    );
}

#[test]
fn source_reference_order_reserves_root_before_index_and_later_arguments() {
    assert_succeeds(
        r#"program Order;
 var Trace: integer := 0;
 var Index: integer := 0;
procedure Change(var Value: integer; Snapshot: array of (integer));
begin Value := Value + Snapshot[0]; end procedure;
function Freeze(): integer; begin Trace := Trace * 10 + 1; return Index; end function;
function Later(): integer; begin Trace := Trace * 10 + 2; Index := 1; return 0; end function;
procedure Use(var Value: integer; Ignored: integer); begin Value := 9; end procedure;
begin
   var Items: array of (integer) := [1, 2];
  Change(var Items[0], Items);
  Use(var Items[Freeze()], Later());
  if (Trace <> 12) or (Items[0] <> 9) or (Items[1] <> 2) then panic('frozen path or snapshot order'); end if;
end program;"#,
    );
}

#[test]
fn source_aliases_cannot_read_or_write_an_exclusive_global_root() {
    for body in ["var Alias: integer := Global;", "Global := 10;"] {
        let source = format!(
            r#"program Alias;  var Global: integer := 1; procedure Change(var Value: integer); begin {body} end procedure; begin Change(var Global); end program;"#
        );
        let error = run_program(&source).expect_err("exclusive alias access must fail");
        assert_eq!(
            error.code,
            fpas_diagnostics::codes::RUNTIME_STORAGE_REFERENCE_CONFLICT
        );
    }
}

#[test]
fn source_later_argument_cannot_mutate_a_reserved_root() {
    let error = run_program(
        r#"program Reservation;
 var Global: integer := 1;
function Mutate(): integer; begin Global := 10; return 0; end function;
procedure Change(var Value: integer; Other: integer); begin Value := Other; end procedure;
begin Change(var Global, Mutate()); end program;"#,
    )
    .expect_err("reserved alias write must fail");
    assert_eq!(
        error.code,
        fpas_diagnostics::codes::RUNTIME_STORAGE_REFERENCE_CONFLICT
    );
}

#[test]
fn source_reference_index_checks_precede_later_argument_evaluation() {
    for source in [
        r#"program T; function Later(): integer; begin panic('later'); return 0; end function; procedure Use(var Value: integer; Ignored: integer); begin null; end procedure; begin  var Items: array of (integer) := [1]; Use(var Items[1], Later()); end program;"#,
        r#"program T; function Later(): integer; begin panic('later'); return 0; end function; procedure Use(var Value: integer; Ignored: integer); begin null; end procedure; begin  var Items: dict of (string, integer) := ['key': 1]; Use(var Items['missing'], Later()); end program;"#,
    ] {
        let error = run_program(source).expect_err("invalid selected storage");
        assert!(!error.message.contains("later"), "{error:?}");
        assert!(
            matches!(
                error.code,
                fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS
                    | fpas_diagnostics::codes::RUNTIME_DICT_KEY_NOT_FOUND
            ),
            "{error:?}"
        );
    }
}

#[test]
fn source_reference_paths_survive_branching_index_expressions() {
    assert_succeeds(
        r#"program T; procedure Use(var Value: integer); begin Value := 42; end procedure; begin  var Items: array of (integer) := [1, 2]; Use(var Items[if false then 0 else 1 end if]); if (Items[0] <> 1) or (Items[1] <> 42) then panic('branch index'); end if; end program;"#,
    );
}

#[test]
fn ordinary_record_functions_and_closures_preserve_var_modes() {
    assert_succeeds(
        r#"program RecordArguments;
type Box = record Item: integer; end record;
procedure BoxApply(Receiver: Box; var Value: integer);
begin Value := Value + Receiver.Item; end procedure;
procedure Increase(var Value: integer);
begin Value := Value + 1; end procedure;
begin
  var Count := 1;
  const Data := Box(Item := 2);
  BoxApply(Data, var Count);
  Increase(var Count);
  const Action := procedure(var Value: integer) begin BoxApply(Data, var Value); end procedure;
  Action(var Count);
  if Count <> 6 then panic('record argument var modes'); end if;
end program;"#,
    );
}
