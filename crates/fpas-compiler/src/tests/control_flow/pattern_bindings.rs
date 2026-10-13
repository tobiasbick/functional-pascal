//! Execution of explicit `const Name` pattern bindings, `_`, and scalar bindings.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/syntax.md`

use super::super::assert_succeeds;

#[test]
fn nonbinding_is_produces_boolean_values_with_single_evaluation_and_short_circuiting() {
    assert_succeeds(
        "program BooleanPatterns;
        type Shape = enum Circle(Radius: integer); Point; end enum;
        var Calls: integer := 0;
        function Next(): Option of integer;
        begin Calls := Calls + 1; return if Calls < 3 then Some(Calls) else None end if; end function;
        function Nested(Value: Result of (Option of Shape, string)): boolean;
        begin return Value is Ok(Some(Shape.Circle(7))); end function;
        function Identity(Flag: boolean): boolean;
        ensures (Returned): Returned = Flag;
        ensures (Flag) or not Flag;
        begin return Flag; end function;
        function Has(Value: Option of integer): boolean;
        requires Value is Some(_);
        ensures (Returned): Returned = (Value is Some(_));
        begin return Value is Some(_); end function;
        begin
          const Hit: boolean := Next() is Some(1);
          if not Hit or Calls <> 1 then panic('stored boolean and single evaluation'); end if;
          if false and (Next() is Some(_)) then panic('and'); end if;
          if true or (Next() is None) then null; else panic('or'); end if;
          if Calls <> 1 then panic('skipped operands'); end if;
          if not Identity((Next() is Some(2)) xor false) then panic('argument and xor'); end if;
          if Calls <> 2 then panic('xor evaluation'); end if;
          const Flags: array of boolean := [Nested(Ok(Some(Shape.Circle(7)))), Nested(Ok(Some(Shape.Circle(8)))), Nested(Ok(None)), Nested(Error('stop'))];
          if not Flags[0] or Flags[1] or Flags[2] or Flags[3] then panic('nested comparisons'); end if;
          if not Has(Some(9)) then panic('restricted contract'); end if;
          Calls := 0;
          while Next() is Some(_) do null; end while;
          if Calls <> 3 then panic('while reevaluation'); end if;
          Calls := 0;
          repeat null; until Next() is None;
          if Calls <> 3 then panic('until reevaluation'); end if;
          case 1 of when 1 if Next() is None: null; else panic('case guard'); end case;
          if Calls <> 4 then panic('guard evaluation'); end if;
        end.",
    );
}

#[test]
fn explicit_bindings_wildcards_and_scalar_bindings_execute() {
    assert_succeeds(
        "\
program PatternBindings;
type Shape = enum
  Rect(Width: integer; Height: integer);
  Point;
end enum;
function Width(S: Shape): integer;
begin
  case S of
    when Shape.Rect(const W, _): return W;
    when Shape.Point: return 0;
  end case;
end function;
function Unwrap(R: result of (integer, string)): integer;
begin
  case R of
    when Ok(const Value): return Value;
    when Error(_): return -1;
  end case;
end function;
function Sign(N: integer): integer;
begin
  case N of
    when const Positive if Positive > 0: return 1;
    when 0: return 0;
    else return -1;
  end case;
end function;
begin
  if Width(Shape.Rect(3, 4)) <> 3 then panic('variant binding'); end if;
  if Width(Shape.Point) <> 0 then panic('fieldless variant'); end if;
  if Unwrap(Ok(7)) <> 7 then panic('result binding'); end if;
  if Unwrap(Error('x')) <> -1 then panic('wildcard payload'); end if;
  if Sign(5) <> 1 then panic('scalar binding'); end if;
  if Sign(-2) <> -1 then panic('scalar else'); end if;
end.",
    );
}

#[test]
fn nested_patterns_and_payload_comparisons_execute_in_arm_order() {
    assert_succeeds(
        "\
program NestedPatterns;
type Shape = enum
  Rect(Width: integer; Height: integer);
  Point;
end enum;
function Classify(R: result of (option of Shape, string)): integer;
begin
  case R of
    when Ok(Some(Shape.Rect(0, _))): return 1;
    when Ok(Some(Shape.Rect(const W, 2))): return W;
    when Ok(Some(_)): return 3;
    when Ok(None): return 4;
    when Error('stop'): return 5;
    when Error(_): return 6;
  end case;
end function;
begin
  if Classify(Ok(Some(Shape.Rect(0, 9)))) <> 1 then panic('nested literal'); end if;
  if Classify(Ok(Some(Shape.Rect(8, 2)))) <> 8 then panic('nested binding'); end if;
  if Classify(Ok(Some(Shape.Point))) <> 3 then panic('nested wildcard'); end if;
  if Classify(Ok(None)) <> 4 then panic('nested none'); end if;
  if Classify(Error('stop')) <> 5 then panic('string comparison'); end if;
  if Classify(Error('other')) <> 6 then panic('error fallback'); end if;
end.",
    );
}

#[test]
fn case_guards_preserve_conditional_chain_evaluation() {
    assert_succeeds(
        "program IsTests;
var Calls: integer := 0;
var Receivers: integer := 0;
function Read(O: Option of integer): Option of integer;
begin Receivers := Receivers + 1; return O; end function;
function Check(Value: integer): boolean;
begin
  Calls := Calls + 1;
  return Value > 1;
end function;
function Probe(O: option of integer): integer;
begin
  return case Read(O) of
    when Some(const Value) if Check(Value): Value;
    when Some(_): 0;
    when None: -1;
  end case;
end function;
begin
  if Probe(Some(5)) <> 5 then panic('bound value'); end if;
  if Probe(Some(1)) <> 0 then panic('failed later condition'); end if;
  if Probe(None) <> -1 then panic('failed pattern'); end if;
  if Calls <> 2 then panic('short circuit'); end if;
  if Receivers <> 3 then panic('selector evaluated again to retrieve payload'); end if;
end.",
    );
}

#[test]
fn closures_capture_comparison_values_before_pattern_bindings_shadow_them() {
    assert_succeeds(
        "program PatternCaptures;
type Pair = enum Both(Left: integer; Right: integer); Empty; end enum;
function MakeCase(): function(Value: Pair): integer;
begin
  const Limit: integer := 3;
  return function(Value: Pair): integer
  begin
    case Value of
      when Pair.Both(const Limit, Limit): return Limit;
      when Pair.Both(_, _), Pair.Empty: return -1;
    end case;
  end function;
end function;
function MakeIs(): function(Value: Pair): integer;
begin
  const Limit: integer := 3;
  return function(Value: Pair): integer
  begin
    return if Value is Pair.Both(_, Limit) then 1 else -1 end if;
  end function;
end function;
function MakeMany(): function(Value: Pair): integer;
begin
  const Limit: integer := 3;
  return function(Value: Pair): integer
  begin
    case Value of
      when Pair.Both(const Limit, 0), Pair.Both(Limit, const Limit): return Limit;
      when Pair.Both(_, _), Pair.Empty: return -1;
    end case;
  end function;
end function;
begin
  const MatchCase: function(Value: Pair): integer := MakeCase();
  const MatchIs: function(Value: Pair): integer := MakeIs();
  const MatchMany: function(Value: Pair): integer := MakeMany();
  if MatchCase(Pair.Both(8, 3)) <> 8 then panic('case capture'); end if;
  if MatchCase(Pair.Both(3, 8)) <> -1 then panic('case outer comparison'); end if;
  if MatchIs(Pair.Both(8, 3)) <> 1 then panic('is capture'); end if;
  if MatchIs(Pair.Both(3, 8)) <> -1 then panic('is outer comparison'); end if;
  if MatchMany(Pair.Both(8, 0)) <> 8 then panic('first label binding'); end if;
  if MatchMany(Pair.Both(3, 8)) <> 8 then panic('later label capture'); end if;
  if MatchMany(Pair.Both(8, 3)) <> -1 then panic('later label outer comparison'); end if;
end.",
    );
}
