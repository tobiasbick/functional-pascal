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
        "unit MyApp.Utils; uses Std.Math; function Clamp(Value: integer; Min: integer; Max: integer): integer; begin if Value < Min then return Min else if Value > Max then return Max else return Value end; function IsBlank(S: string): boolean; begin return Length(Trim(S)) = 0 end;",
        include_str!("golden/unit_clamp.expected.fpas"),
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
fn wrapped_record_literal() {
    common::assert_golden(
        "wrapped_record",
        "program T; type Config = record Host: string; Port: integer; Retries: integer; TimeoutSeconds: integer; end; begin var C: Config := record Host := 'api.example.com'; Port := 443; Retries := 5; TimeoutSeconds := 30; end; end.",
        include_str!("golden/wrapped_record.expected.fpas"),
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
