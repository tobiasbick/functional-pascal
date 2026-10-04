//! Golden-file output tests ([`docs/pascal/tools/fmt-style.md`](../../docs/pascal/tools/fmt-style.md)).

#![allow(clippy::expect_used)]

mod common;

#[test]
fn hello_minimal() {
    common::assert_golden(
        "hello_minimal",
        r#"program Hello; begin WriteLn('Hello, World!'); end program;"#,
        include_str!("golden/hello_minimal.expected.fpas"),
    );
}

#[test]
fn hello_uses() {
    common::assert_golden(
        "hello_uses",
        r#"program Hello;  uses Std.Console as Console; begin Console.WriteLn('Hello, World!'); end program;"#,
        include_str!("golden/hello_uses.expected.fpas"),
    );
}

#[test]
fn unit_clamp() {
    common::assert_golden(
        "unit_clamp",
        r#"unit MyApp.Utils;  uses Std.Math as Math; public function Clamp(Value: integer; Min: integer; Max: integer): integer; begin if Value < Min then return Min; else if Value > Max then return Max; else return Value; end if; end if; end function; function IsBlank(S: string): boolean; begin return Length(Trim(S)) = 0; end function;
end unit;
"#,
        include_str!("golden/unit_clamp.expected.fpas"),
    );
}

#[test]
fn record_member_visibility() {
    common::assert_golden(
        "record_visibility",
        r#"unit Demo.Counter;

public type Counter = record
  Value: integer;
  public Step: integer;
  public Changed: Option of (procedure()) := Option.None;
end record;

function Hidden(Receiver: Counter): integer;
begin
  return Receiver.Value;
end function;

public function CounterCreate(): Counter;
begin
  return Counter(Value := 0, Step := 1);
end function;

public function CounterCurrent(Receiver: Counter): integer;
begin
  return Hidden(Receiver);
end function;

end unit;
"#,
        include_str!("golden/record_visibility.expected.fpas"),
    );
}

#[test]
fn long_uses() {
    common::assert_golden(
        "long_uses",
        r#"program LongUses;  uses Std.Console as Console; uses Std.Conv as Conv; uses Std.Arrays as Arrays; uses Std.Dictionaries as Dictionaries; uses Std.Options as Options; uses Std.Results as Results; uses Std.String as String; uses MyApp.Very.Long.Namespace.One as One; uses MyApp.Very.Long.Namespace.Two as Two; begin Console.WriteLn('ok'); end program;"#,
        include_str!("golden/long_uses.expected.fpas"),
    );
}

#[test]
fn short_record_construction_uses_named_fields() {
    common::assert_golden(
        "short_record",
        r#"program T;

type Point = record
  X: integer;
  Y: integer;
end record;

begin
  const A: Point := Point(X := 3, Y := 4);
end program;
"#,
        include_str!("golden/short_record.expected.fpas"),
    );
}

#[test]
fn logical_block_spacing() {
    common::assert_golden(
        "logical_block_spacing",
        r#"program T;

type Point = record
  X: integer;
  Y: integer;
end record;

begin
  const A: Point := Point(X := 3, Y := 4);
  const B: Point := Point(X := 10, Y := 20);
  const UpdatedB: Point := B with X := 11; end with;

  A.Print();
  if Ready then
    Save();
  end if;

  // present result
  Present();
  if NeedsCount then
    Prepare();
  end if;
  const Count: integer := 1;
  WriteLn(Count);
  if Done then
    Finish();
  end if;
end program;
"#,
        include_str!("golden/logical_block_spacing.expected.fpas"),
    );
}

#[test]
fn wrapped_parenthesized_comparisons_preserve_full_expression() {
    common::assert_golden(
        "wrapped_parenthesized_comparisons",
        r#"program T; begin const InsideHorizontalBounds: boolean := (MouseEvent.mouse_x > ButtonBounds.x) and (MouseEvent.mouse_x <= ButtonBounds.x + ButtonBounds.width); end program;"#,
        r#"program T;

begin
  const InsideHorizontalBounds: boolean := (MouseEvent.mouse_x > ButtonBounds.x) and
                                           (MouseEvent.mouse_x <= ButtonBounds.x + ButtonBounds.width);
end program;
"#,
    );
}

#[test]
fn comments_unit_declaration_docs() {
    common::assert_golden(
        "comments_unit",
        r#"// Unit doc.
unit Demo;

// field doc
 var Count: integer := 0;
end unit;
"#,
        include_str!("golden/comments_unit.expected.fpas"),
    );
}

#[test]
fn comments_program_uses_begin_body_and_trailing() {
    common::assert_golden(
        "comments_program",
        r#"program T;
// before uses
 uses Std.Console as Console;

// before begin
begin
  // setup
  Console.WriteLn('ok'); // trail
end program; // tail"#,
        include_str!("golden/comments_program.expected.fpas"),
    );
}

#[test]
fn comments_before_begin_and_statement() {
    common::assert_golden(
        "comments_before_body",
        r#"program T;
// before begin
begin
  // in body
  WriteLn('ok');
end program;"#,
        include_str!("golden/comments_before_body.expected.fpas"),
    );
}

#[test]
fn postfix_chaining_compact() {
    common::assert_golden(
        "postfix_chaining",
        r#"program CompactPostfix; begin const X: integer := Factory.Create().Transform(2).Value; end program;"#,
        include_str!("golden/postfix_chaining.expected.fpas"),
    );
    common::assert_round_trip(
        "postfix_chaining_round_trip",
        r#"program CompactPostfix; begin const X: integer := Factory.Create().Transform(2).Value; end program;"#,
    );
}

#[test]
fn postfix_chaining_wraps_long_chain() {
    let source = r#"program T; begin const X: integer := VeryLongFactoryName.CreateVeryLongThing().TransformWithVeryLongName(VeryLongArgumentAlpha).ScaleWithAnotherLongName(VeryLongArgumentBeta).Value; end program;"#;
    let (unit, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:?}");
    let formatted = fpas_fmt::format_source(source, &unit).expect("matching source and AST");
    assert!(
        formatted.contains("\n") && formatted.contains('.'),
        "expected wrapped postfix suffixes: {formatted}"
    );
    assert!(
        formatted
            .lines()
            .any(|line| line.trim_start().starts_with('.')),
        "expected continuation line starting with `.`: {formatted}"
    );
    assert!(
        formatted.contains(".TransformWithVeryLongName(VeryLongArgumentAlpha)"),
        "{formatted}"
    );
    assert!(
        formatted.contains(".ScaleWithAnotherLongName(VeryLongArgumentBeta)"),
        "{formatted}"
    );
    common::assert_round_trip("postfix_chaining_wrapped", &formatted);
}

#[test]
fn closure_literal_round_trips() {
    common::assert_round_trip(
        "closure_compact",
        r#"program T; begin const F: procedure() := procedure() begin null; end procedure; end program;"#,
    );
    common::assert_round_trip(
        "closure_multiline",
        r#"program T;
begin
  const Add: function(Value: integer): integer :=
    function(Value: integer): integer
    begin
      return Value + 1;
    end function;
end program;"#,
    );
}

#[test]
fn postfix_chaining_round_trips_field_index_mixture() {
    common::assert_round_trip(
        "postfix_field_index_mixture",
        r#"program T; begin const X: integer := Factory.Create().Items[0].Value; end program;"#,
    );
}
