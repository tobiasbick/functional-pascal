//! Golden-file output tests ([`docs/pascal/tools/fmt-style.md`](../../docs/pascal/tools/fmt-style.md)).

mod common;

#[test]
fn hello_minimal() {
    common::assert_golden(
        "hello_minimal",
        "program Hello; begin WriteLn('Hello, World!') end.",
        include_str!("golden/hello_minimal.expected.fpas"),
    );
}

#[test]
fn hello_uses() {
    common::assert_golden(
        "hello_uses",
        "program Hello; uses Std.Console; begin WriteLn('Hello, World!') end.",
        include_str!("golden/hello_uses.expected.fpas"),
    );
}

#[test]
fn unit_clamp() {
    common::assert_golden(
        "unit_clamp",
        "unit MyApp.Utils; uses Std.Math; public function Clamp(Value: integer; Min: integer; Max: integer): integer; begin if Value < Min then return Min else if Value > Max then return Max else return Value end; function IsBlank(S: string): boolean; begin return Length(Trim(S)) = 0 end;",
        include_str!("golden/unit_clamp.expected.fpas"),
    );
}

#[test]
fn record_member_visibility() {
    common::assert_golden(
        "record_visibility",
        "unit Demo.Counter; public type Counter = record Value: integer; public Step: integer; function Hidden(Self: Counter): integer; begin return Self.Value end; public static function Create(): Counter; begin return record Value := 0; Step := 1; end end; public property Current: integer read Hidden; public event Changed: procedure() read ReadChanged write WriteChanged; end;",
        include_str!("golden/record_visibility.expected.fpas"),
    );
}

#[test]
fn long_uses() {
    common::assert_golden(
        "long_uses",
        "program LongUses; uses Std.Console, Std.Conv, Std.Array, Std.Dict, Std.Option, Std.Result, Std.String, MyApp.Very.Long.Namespace.One, MyApp.Very.Long.Namespace.Two; begin WriteLn('ok') end.",
        include_str!("golden/long_uses.expected.fpas"),
    );
}

#[test]
fn short_record_literal_is_multiline() {
    common::assert_golden(
        "short_record",
        "program T; type Point = record X: integer; Y: integer; end; begin var A: Point := record X := 3; Y := 4; end; end.",
        include_str!("golden/short_record.expected.fpas"),
    );
}

#[test]
fn wrapped_parenthesized_comparisons_preserve_full_expression() {
    common::assert_golden(
        "wrapped_parenthesized_comparisons",
        "program T; begin var InsideHorizontalBounds: boolean := (MouseEvent.mouse_x > ButtonBounds.x) and (MouseEvent.mouse_x <= ButtonBounds.x + ButtonBounds.width); end.",
        "program T;\n\nbegin\n  var InsideHorizontalBounds: boolean := (MouseEvent.mouse_x > ButtonBounds.x) and\n                                         (MouseEvent.mouse_x <= ButtonBounds.x + ButtonBounds.width)\nend.\n",
    );
}

#[test]
fn comments_unit_declaration_docs() {
    common::assert_golden(
        "comments_unit",
        "/// Unit doc.\nunit Demo;\n\n{ field doc }\nmutable var Count: integer := 0;\n",
        include_str!("golden/comments_unit.expected.fpas"),
    );
}

#[test]
fn comments_program_uses_begin_body_and_trailing() {
    common::assert_golden(
        "comments_program",
        "program T;\n{ before uses }\nuses Std.Console;\n\n{ before begin }\nbegin\n  // setup\n  WriteLn('ok') // trail\nend. // tail",
        include_str!("golden/comments_program.expected.fpas"),
    );
}

#[test]
fn comments_brace_and_paren_star_blocks() {
    common::assert_golden(
        "comments_block_styles",
        "program T;\n(* before begin *)\nbegin\n  { in body }\n  WriteLn('ok')\nend.",
        include_str!("golden/comments_block_styles.expected.fpas"),
    );
}

#[test]
fn postfix_chaining_compact() {
    common::assert_golden(
        "postfix_chaining",
        "program CompactPostfix; begin var X: integer := Factory.Create().Transform(2).Value; end.",
        include_str!("golden/postfix_chaining.expected.fpas"),
    );
    common::assert_round_trip(
        "postfix_chaining_round_trip",
        "program CompactPostfix; begin var X: integer := Factory.Create().Transform(2).Value; end.",
    );
}

#[test]
fn postfix_chaining_wraps_long_chain() {
    let source = "program T; begin var X: integer := VeryLongFactoryName.CreateVeryLongThing().TransformWithVeryLongName(VeryLongArgumentAlpha).ScaleWithAnotherLongName(VeryLongArgumentBeta).Value; end.";
    let (unit, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:?}");
    let formatted = fpas_fmt::format_source(source, &unit);
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
    common::assert_round_trip("postfix_chaining_wrapped", &formatted);
}

#[test]
fn closure_literal_round_trips() {
    common::assert_round_trip(
        "closure_compact",
        "program T; begin var F: procedure() := procedure() begin end; end.",
    );
    common::assert_round_trip(
        "closure_multiline",
        "program T;\nbegin\n  var Add: function(Value: integer): integer :=\n    function(Value: integer): integer\n    begin\n      return Value + 1\n    end\nend.",
    );
}

#[test]
fn postfix_chaining_round_trips_field_index_mixture() {
    common::assert_round_trip(
        "postfix_field_index_mixture",
        "program T; begin var X: integer := Factory.Create().Items[0].Value; end.",
    );
}
