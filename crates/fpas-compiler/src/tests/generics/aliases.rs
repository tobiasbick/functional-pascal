use super::*;

// ═══════════════════════════════════════════════════════════════
// TYPE ALIASES — positive
// ═══════════════════════════════════════════════════════════════

#[test]
fn generic_type_alias() {
    let out = compile_and_run(
        "\
program GenericAlias;
uses Std.Console;

type
  Box<T> = record
    Value: T;
  end;
  IntBox = Box of integer;

begin
  var B: IntBox := record Value := 7; end;
  WriteLn(B.Value)
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn generic_type_alias_string() {
    let out = compile_and_run(
        "\
program GenericAliasStr;
uses Std.Console;

type
  Box<T> = record
    Value: T;
  end;
  StrBox = Box of string;

begin
  var B: StrBox := record Value := 'wow'; end;
  WriteLn(B.Value)
end.",
    );
    assert_eq!(out.lines, vec!["wow"]);
}

#[test]
fn generic_type_alias_for_enum() {
    let out = compile_and_run(
        "\
program GenericAliasEnum;
uses Std.Console;

type
  Maybe<T> = enum
    Just(Value: T);
    Nothing;
  end;
  MaybeInt = Maybe of integer;

begin
  var M: MaybeInt := Maybe.Just(5);
  case M of
    Maybe.Just(V): WriteLn(V);
    Maybe.Nothing: WriteLn('nope')
  end
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}
