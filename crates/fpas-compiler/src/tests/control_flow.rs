use super::*;

mod counting;
mod ordinal;
mod repetition;

#[test]
fn for_in_array_and_dictionary_execute() {
    assert_succeeds(
        "\
program RegisterForIn;
uses Std.Console, Std.Conv, Std.Dict;
begin
  mutable var Sum: integer := 0;
  for Value: integer in [1, 2, 3] do Sum := Sum + Value;
  var Values: dict of string to integer := ['a': 4, 'b': 5];
  for Key: string in Values do
  begin
    WriteLn(IntToStr(Values[Key]));
    Sum := Sum + Values[Key]
  end;
  if Sum <> 15 then panic('for-in mismatch')
end.",
    );
}

#[test]
fn scalar_locals_temporaries_and_operations_execute() {
    let execution = assert_succeeds(
        "\
program RegisterScalars;
begin
  mutable var I: integer := 7;
  mutable var R: real := 1.5;
  mutable var S: string := 'ab';
  mutable var B: boolean := true;
  I := ((I * 3) - 1) div 2;
  R := (R + 2) / 2;
  S := S + 'cd';
  B := (B and not false) xor false;
  if (I <> 10) or (R <> 1.75) or (S <> 'abcd') or (not B) then
    panic('scalar mismatch')
end.",
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}

#[test]
fn nested_while_repeat_for_break_and_continue_execute() {
    assert_succeeds(
        "\
program RegisterLoops;
begin
  mutable var Sum: integer := 0;
  mutable var I: integer := 0;
  while I < 4 do
  begin
    I := I + 1;
    if I = 2 then continue;
    for J: integer := 3 downto 1 do
    begin
      if J = 2 then continue;
      Sum := Sum + I * J;
      if Sum > 40 then break
    end
  end;
  repeat
    Sum := Sum - 1;
    if Sum = 30 then break
  until Sum < 0;
  if Sum <> 30 then panic('loop mismatch')
end.",
    );
}

#[test]
fn scalar_case_values_ranges_guards_and_else_execute() {
    assert_succeeds(
        "\
program RegisterCase;
begin
  mutable var Score: integer := 0;
  var I: integer := 5;
  case I of
    Candidate if Candidate < 0: Score := 99;
    1..3: Score := 1;
    5 if I > 5: Score := 2;
    5: Score := 3
  else
    Score := 4
  end;
  var S: string := 'beta';
  case S of
    'alpha': Score := 10;
    'beta': Score := Score + 4
  else
    Score := 20
  end;
  var Flag: boolean := true;
  case Flag of
    false: Score := 100;
    true: Score := Score + 5
  end;
  if Score <> 12 then panic('case mismatch')
end.",
    );
}

#[test]
fn mixed_numeric_comparisons_and_integer_edges_execute() {
    assert_succeeds(
        "\
program RegisterNumeric;
begin
  mutable var X: integer := 9223372036854775807;
  X := X + 1;
  var Bits: integer := (12 and 10) or (3 xor 1);
  var Shifted: integer := (1 shl 5) shr 2;
  if (X <> -9223372036854775807 - 1) or (Bits <> 10) or (Shifted <> 8) then
    panic('integer mismatch');
  if not (2 < 2.5) then panic('mixed comparison mismatch')
end.",
    );
}
