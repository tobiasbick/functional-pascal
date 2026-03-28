use super::*;

// ═══════════════════════════════════════════════════════════════
// GENERIC ENUMS — positive
// ═══════════════════════════════════════════════════════════════

#[test]
fn generic_enum_either() {
    let out = compile_and_run(
        "\
program GenericEither;
uses Std.Console, Std.Conv;

type
  Either<L, R> = enum
    Left(Value: L);
    Right(Value: R);
  end;

begin
  var E: Either of string, integer := Either.Right(42);
  case E of
    Either.Left(V): WriteLn('left: ' + V);
    Either.Right(V): WriteLn('right: ' + IntToStr(V))
  end
end.",
    );
    assert_eq!(out.lines, vec!["right: 42"]);
}

#[test]
fn generic_enum_either_left_branch() {
    let out = compile_and_run(
        "\
program GenericEitherLeft;
uses Std.Console;

type
  Either<L, R> = enum
    Left(Value: L);
    Right(Value: R);
  end;

begin
  var E: Either of string, integer := Either.Left('hello');
  case E of
    Either.Left(V): WriteLn(V);
    Either.Right(V): WriteLn('right')
  end
end.",
    );
    assert_eq!(out.lines, vec!["hello"]);
}

#[test]
fn generic_enum_with_fieldless_variant() {
    let out = compile_and_run(
        "\
program GenericMaybe;
uses Std.Console;

type
  Maybe<T> = enum
    Just(Value: T);
    Nothing;
  end;

begin
  var M: Maybe of string := Maybe.Just('hi');
  case M of
    Maybe.Just(V): WriteLn(V);
    Maybe.Nothing: WriteLn('nothing')
  end
end.",
    );
    assert_eq!(out.lines, vec!["hi"]);
}

#[test]
fn generic_enum_fieldless_variant_chosen() {
    let out = compile_and_run(
        "\
program GenericMaybeNone;
uses Std.Console;

type
  Maybe<T> = enum
    Just(Value: T);
    Nothing;
  end;

begin
  var M: Maybe of integer := Maybe.Nothing;
  case M of
    Maybe.Just(V): WriteLn('just');
    Maybe.Nothing: WriteLn('nothing')
  end
end.",
    );
    assert_eq!(out.lines, vec!["nothing"]);
}

#[test]
fn generic_enum_multiple_fields_in_variant() {
    let out = compile_and_run(
        "\
program GenericKV;
uses Std.Console, Std.Conv;

type
  Entry<K, V> = enum
    Pair(Key: K; Val: V);
    Empty;
  end;

begin
  var E: Entry of string, integer := Entry.Pair('x', 42);
  case E of
    Entry.Pair(K, V): WriteLn(K + '=' + IntToStr(V));
    Entry.Empty: WriteLn('empty')
  end
end.",
    );
    assert_eq!(out.lines, vec!["x=42"]);
}
