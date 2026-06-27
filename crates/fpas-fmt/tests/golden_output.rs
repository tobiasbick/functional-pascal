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

#[test]
fn comments_unit_declaration_docs() {
    common::assert_golden(
        "comments_unit",
        "/// Unit doc.\nunit Demo;\n\n{ field doc }\nprivate mutable var Count: integer := 0;\n",
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
