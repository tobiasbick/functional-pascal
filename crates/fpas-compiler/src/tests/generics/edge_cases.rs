use super::*;

// ═══════════════════════════════════════════════════════════════
// EDGE CASES — positive
// ═══════════════════════════════════════════════════════════════

#[test]
fn generic_single_letter_type_params() {
    let out = compile_and_run(
        "\
program SingleLetter;
uses Std.Console;

function Id<X>(V: X): X;
begin
  return V
end;

begin
  WriteLn(Id(7))
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn generic_long_type_param_name() {
    let out = compile_and_run(
        "\
program LongParamName;
uses Std.Console;

function Id<Element>(V: Element): Element;
begin
  return V
end;

begin
  WriteLn(Id('test'))
end.",
    );
    assert_eq!(out.lines, vec!["test"]);
}

#[test]
fn generic_type_param_same_name_different_types() {
    // Two generic types both use T but independently
    let out = compile_and_run(
        "\
program IndependentT;
uses Std.Console;

type
  BoxA<T> = record Value: T; end;
  BoxB<T> = record Item: T; end;

begin
  var A: BoxA of integer := record Value := 1; end;
  var B: BoxB of string := record Item := 'hi'; end;
  WriteLn(A.Value);
  WriteLn(B.Item)
end.",
    );
    assert_eq!(out.lines, vec!["1", "hi"]);
}

#[test]
fn generic_record_with_option_field() {
    let out = compile_and_run(
        "\
program GenericOptionField;
uses Std.Console;

type
  Container<T> = record
    Value: Option of T;
  end;

begin
  var C: Container of integer := record Value := Some(42); end;
  case C.Value of
    Some(V): WriteLn(V);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn generic_record_with_result_field() {
    let out = compile_and_run(
        "\
program GenericResultField;
uses Std.Console;

type
  Holder<T> = record
    Value: Result of T, string;
  end;

begin
  var H: Holder of integer := record Value := Ok(99); end;
  case H.Value of
    Ok(V): WriteLn(V);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["99"]);
}

#[test]
fn generic_three_type_params() {
    let out = compile_and_run(
        "\
program ThreeParams;
uses Std.Console;

function Pick<A, B, C>(X: A; Y: B; Z: C): C;
begin
  return Z
end;

begin
  WriteLn(Pick(1, 'two', true))
end.",
    );
    assert_eq!(out.lines, vec!["true"]);
}
