use super::*;

mod boolean;
mod counting;
mod ordinal;
mod repetition;

#[test]
fn for_in_array_and_dictionary_execute() {
    assert_succeeds(
        "program RegisterForIn;\nuses Std.Console, Std.Conv;\nbegin\n  var Sum: integer := 0;\n  for Value: integer in [1, 2, 3] do Sum := Sum + Value; end for;\n  const Values: dict of string to integer := ['a': 4, 'b': 5];\n  for Key: string in Values do\n  begin\n    WriteLn(IntToStr(Values[Key]));\n    Sum := Sum + Values[Key];\n  end; end for;\n  if Sum <> 15 then panic('for-in mismatch'); end if;\nend.",
    );
}

#[test]
fn scalar_locals_temporaries_and_operations_execute() {
    let execution = assert_succeeds(
        "\
program RegisterScalars;
begin
  var I: integer := 7;
  var R: real := 1.5;
  var S: string := 'ab';
  var B: boolean := true;
  I := ((I * 3) - 1) div 2;
  R := (R + 2) / 2;
  S := S + 'cd';
  B := (B and not false) xor false;
  if (I <> 10) or (R <> 1.75) or (S <> 'abcd') or (not B) then
    panic('scalar mismatch'); end if;
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
  var Sum: integer := 0;
  var I: integer := 0;
  while I < 4 do
  begin
    I := I + 1;
    if I = 2 then continue; end if;
    for J: integer := 3 downto 1 do
    begin
      if J = 2 then continue; end if;
      Sum := Sum + I * J;
      if Sum > 40 then break; end if;
    end; end for;
  end; end while;
  repeat
    Sum := Sum - 1;
    if Sum = 30 then break; end if;
  until Sum < 0;
  if Sum <> 30 then panic('loop mismatch'); end if;
end.",
    );
}

#[test]
fn scalar_case_values_ranges_guards_and_else_execute() {
    assert_succeeds(
        "\
program RegisterCase;
begin
  var Score: integer := 0;
  const I: integer := 5;
  case I of
    when Candidate if Candidate < 0: Score := 99;
    when 1..3: Score := 1;
    when 5 if I > 5: Score := 2;
    when 5: Score := 3;
  else
    Score := 4;
  end case;
  const S: string := 'beta';
  case S of
    when 'alpha': Score := 10;
    when 'beta': Score := Score + 4;
  else
    Score := 20;
  end case;
  const Flag: boolean := true;
  case Flag of
    when false: Score := 100;
    when true: Score := Score + 5;
  end case;
  if Score <> 12 then panic('case mismatch'); end if;
end.",
    );
}

#[test]
fn mixed_numeric_comparisons_and_integer_edges_execute() {
    assert_succeeds(
        "\
program RegisterNumeric;
uses Std.Bits;
begin
  var X: integer := 9223372036854775807;
  X := X + 1;
  const Bits: integer := BitOr(BitAnd(12, 10), BitXor(3, 1));
  const Shifted: integer := ShiftRight(ShiftLeft(1, 5), 2);
  if (X <> -9223372036854775807 - 1) or (Bits <> 10) or (Shifted <> 8) then
    panic('integer mismatch'); end if;
  if not (2 < 2.5) then panic('mixed comparison mismatch'); end if;
end.",
    );
}
