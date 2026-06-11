//! Golden-file output tests ([`docs/future/formater/style.md`](../../docs/future/formater/style.md)).

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
