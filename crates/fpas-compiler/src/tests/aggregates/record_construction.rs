//! Runtime order, default binding, and contextual lowering for typed record calls.

use super::super::assert_succeeds;

#[test]
fn supplied_fields_run_once_in_written_order_then_defaults_in_declaration_order() {
    assert_succeeds("program T;
      var Trace: integer := 0;
      function Mark(Value: integer): integer; begin Trace := Trace * 10 + Value; return Value; end function;
      type Data = record A: integer := Mark(1); B: integer := Mark(2); C: integer := Mark(3); D: integer := Mark(4); end record;
      begin
        const P: Data := Data(D := Mark(8), B := Mark(6));
        if (Trace <> 8613) or (P.A <> 1) or (P.B <> 6) or (P.C <> 3) or (P.D <> 8) then panic('constructor order'); end if;
        Trace := 0;
        // Stage defaults and explicit fields in the migrated declaration order.
        const First: integer := Mark(1);
        const Second: integer := Mark(6);
        const Third: integer := Mark(3);
        const Fourth: integer := Mark(8);
        const Migrated: Data := Data(A := First, B := Second, C := Third, D := Fourth);
        if (Trace <> 1638) or (Migrated.A <> 1) or (Migrated.B <> 6) or (Migrated.C <> 3) or (Migrated.D <> 8) then panic('migration order changed'); end if;
      end.");
}

#[test]
fn aliases_nested_construction_and_expected_field_types_execute() {
    assert_succeeds("program T;
      type Point = record X: integer; Y: integer := 2; end record;
      type Position = Point;
      type Box = record Value: Point; Items: array of Point; end record;
      type Empty = record end record;
      function Make<T>(Value: T): Box; begin return Box(Items := [], Value := Position(X := 5)); end function;
      begin
        const E: Empty := Empty();
        const B: Box := Box(Items := [Point(X := 3)], Value := Position(Y := 4, X := 1));
        const G: Box := Make('x');
        var V: Point := Point(X := 0);
        V := Position(X := 7);
        if (B.Value.X <> 1) or (B.Value.Y <> 4) or (B.Items[0].Y <> 2) or (G.Value.X <> 5) or (G.Items.Length() <> 0) or (V.Y <> 2) then panic('typed values'); end if;
        if Point(X := 9).X <> 9 then panic('postfix constructor'); end if;
      end.");
}

#[test]
fn retained_defaults_can_contain_nested_constructors_and_named_routine_calls() {
    assert_succeeds("program T;
      function Sub(Left: integer; Right: integer): integer; begin return Left - Right; end function;
      type Point = record X: integer; end record;
      type Box = record P: Point := Point(X := Sub(Right := 2, Left := 9)); Items: array of Point := [Point(X := 4)]; end record;
      begin const B: Box := Box(); if (B.P.X <> 7) or (B.Items[0].X <> 4) then panic('nested default metadata'); end if; end.");
}

#[test]
fn field_defaults_keep_declaration_scope_when_callers_shadow_names() {
    assert_succeeds("program T;
      var Value: integer := 7;
      function GetValue(): integer; begin return Value; end function;
      type Box = record A: integer := Value; B: integer := GetValue(); end record;
      function Probe(Value: integer): Box;
        function GetValue(): integer; begin return 99; end function;
      begin return Box(); end function;
      begin const B: Box := Probe(42); if (B.A <> 7) or (B.B <> 7) then panic('default captured caller'); end if; end.");
}

#[test]
fn ordinary_record_returning_routines_and_shadowing_function_values_keep_their_calls() {
    assert_succeeds("program T;
      type Point = record X: integer; end record;
      function Make(Value: integer): Point; begin return Point(X := Value + 1); end function;
      function Apply(Point: function(Value: integer): Point): Point; begin return Point(3); end function;
      begin const P: Point := Apply(Make); if P.X <> 4 then panic('ordinary call became constructor'); end if; end.");
}

#[test]
fn try_preserves_earlier_fields_and_stops_before_later_arguments_or_defaults() {
    assert_succeeds("program T;
      var Trace: integer := 0;
      function Mark(Value: integer): integer; begin Trace := Trace * 10 + Value; return Value; end function;
      function GetValue(Value: integer; FailAt: integer): result of integer, string;
      begin discard Mark(Value); if Value = FailAt then return Error('failed'); end if; return Ok(Value); end function;
      type Pair = record X: integer; Y: integer; Z: integer := Mark(3); end record;
      function Probe(FailAt: integer): result of integer, string;
      begin const P: Pair := Pair(Y := try GetValue(1, FailAt), X := try GetValue(2, FailAt)); return Ok(P.X * 10 + P.Y); end function;
      begin
        if (Probe(0) <> Ok(21)) or (Trace <> 123) then panic('success order'); end if;
        Trace := 0; if (Probe(1) <> Error('failed')) or (Trace <> 1) then panic('first error order'); end if;
        Trace := 0; if (Probe(2) <> Error('failed')) or (Trace <> 12) then panic('second error order'); end if;
      end.");
}

#[test]
fn callable_defaults_and_explicit_fields_retain_capture_proofs() {
    assert_succeeds("program T;
      type Holder = record Callback: function(Value: integer): integer := function(Value: integer): integer begin return Value + 1; end function; end record;
      begin
        const H: Holder := Holder();
        if H.Callback(4) <> 5 then panic('callable default'); end if;
        const Offset: integer := 3;
        const K: Holder := Holder(Callback := function(Value: integer): integer begin return Value + Offset; end function);
        discard K;
      end.");
}
