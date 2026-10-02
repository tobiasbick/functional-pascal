//! `program` and `unit` compilation units.

use fpas_parser::{Import, Program, Unit};

use crate::comments::{
    CommentMap, emit_leading_comments, emit_trailing_comments, emit_trailing_end_comments,
};

use super::Emitter;
use super::decl::emit_decls;
use super::stmt::emit_stmts_in_block;
use super::types::emit_qualified_id;

/// Formats a `program` compilation unit.
#[must_use]
pub(crate) fn format_program(program: &Program, comments: &CommentMap) -> String {
    let mut emitter = Emitter::new();
    emit_program(&mut emitter, program, comments);
    emitter.finish()
}

/// Formats a `unit` compilation unit.
#[must_use]
pub(crate) fn format_unit(unit: &Unit, comments: &CommentMap) -> String {
    let mut emitter = Emitter::new();
    emit_unit(&mut emitter, unit, comments);
    emitter.finish()
}

fn emit_program(emitter: &mut Emitter, program: &Program, comments: &CommentMap) {
    emit_leading_comments(emitter, comments, program.span.offset, true);
    emitter.write(&format!("program {};", program.name));
    finish_header_line(emitter, comments, program.span.offset);
    emitter.blank_line();
    emit_optional_uses(emitter, &program.uses, comments);
    if !program.declarations.is_empty() {
        emit_decls(emitter, &program.declarations, comments);
        emitter.blank_line();
    }
    if let Some(anchor) = comments.body_anchor(program.span.offset) {
        emit_leading_comments(emitter, comments, anchor, false);
    }
    emitter.writeln("begin");
    emitter.with_indent(|inner| emit_stmts_in_block(inner, &program.body, comments));
    emitter.write_current_indent();
    emitter.write("end program;");
    emit_trailing_comments(emitter, comments, program.span.offset);
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
    emit_trailing_end_comments(emitter, comments);
}

fn emit_unit(emitter: &mut Emitter, unit: &Unit, comments: &CommentMap) {
    emit_leading_comments(emitter, comments, unit.span.offset, true);
    emitter.write("unit ");
    emit_qualified_id(emitter, &unit.name);
    emitter.write(";");
    finish_header_line(emitter, comments, unit.span.offset);
    emitter.blank_line();
    emit_optional_uses(emitter, &unit.uses, comments);
    if !unit.declarations.is_empty() {
        emit_decls(emitter, &unit.declarations, comments);
        emitter.blank_line();
    }
    emitter.write_current_indent();
    emitter.write("end unit;");
    emit_trailing_comments(emitter, comments, unit.span.offset);
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
    emit_trailing_end_comments(emitter, comments);
}

fn emit_optional_uses(emitter: &mut Emitter, uses: &[Import], comments: &CommentMap) {
    if uses.is_empty() {
        return;
    }
    for unit_name in uses {
        let offset = unit_name.span.offset;
        emit_leading_comments(emitter, comments, offset, false);
        emitter.write_current_indent();
        emitter.write("uses ");
        emitter.write(&unit_name.parts.join("."));
        emitter.write(" as ");
        emitter.write(&unit_name.alias);
        emitter.write(";");
        emit_trailing_comments(emitter, comments, offset);
        if !emitter.ends_with_newline() {
            emitter.write_line_end();
        }
    }
    emitter.blank_line();
}

fn finish_header_line(emitter: &mut Emitter, comments: &CommentMap, owner_start: usize) {
    if let Some(anchor) = comments.header_anchor(owner_start) {
        emit_trailing_comments(emitter, comments, anchor);
    }
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
}

#[cfg(test)]
mod tests {
    use super::format_program;
    use crate::comments::CommentMap;
    use crate::format_source;
    use fpas_parser::parse_compilation_unit;

    fn parse_and_format(source: &str) -> String {
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        format_source(source, &unit).expect("matching source and AST")
    }

    #[test]
    fn minimal_program() {
        let formatted =
            parse_and_format(r#"program Hello; begin WriteLn('Hello, World!'); end program;"#);
        assert_eq!(
            formatted,
            r#"program Hello;

begin
  WriteLn('Hello, World!');
end program;
"#
        );
    }

    #[test]
    fn program_with_uses() {
        let formatted = parse_and_format(
            r#"program Hello;  uses Std.Console as Console; begin Console.WriteLn('Hello, World!'); end program;"#,
        );
        assert_eq!(
            formatted,
            r#"program Hello;

uses Std.Console as Console;

begin
  Console.WriteLn('Hello, World!');
end program;
"#
        );
    }

    #[test]
    fn unit_clamp_preserves_branch_lists() {
        let source = r#"unit MyApp.Utils;  uses Std.Math as Math; function Clamp(Value: integer; Min: integer; Max: integer): integer; begin if Value < Min then return Min; else if Value > Max then return Max; else return Value; end if; end if; end function; function IsBlank(S: string): boolean; begin return Length(Trim(S)) = 0; end function;
end unit;
"#;
        let formatted = parse_and_format(source);
        assert!(formatted.starts_with(
            r#"unit MyApp.Utils;

uses Std.Math as Math;

"#
        ));
        assert!(formatted.contains("if Value < Min then\n    return Min;\n"));
        assert!(formatted.contains("function IsBlank"));
    }

    #[test]
    fn program_type_then_begin() {
        let formatted = parse_and_format(
            r#"program T;  type Point = record X: integer; Y: integer; end record; begin var P: Point := record X := 1; Y := 2; end record; end program;"#,
        );
        assert!(formatted.contains("type Point = record\n"));
        assert!(formatted.contains("end record;\n\nbegin\n"));
    }

    #[test]
    fn array_literal_short_stays_single_line() {
        let formatted = parse_and_format(
            r#"program T; begin var Words: array of string := ['red', 'green', 'blue']; end program;"#,
        );
        assert!(
            formatted.contains("['red', 'green', 'blue']"),
            "formatted:\n{formatted}"
        );
    }

    #[test]
    fn long_uses_clause_wraps() {
        let formatted = parse_and_format(
            r#"program LongUses;  uses Std.Console as Console; uses Std.Conv as Conv; uses Std.Arrays as Arrays; uses Std.Dictionaries as Dictionaries; uses Std.Options as Options; uses Std.Results as Results; uses Std.String as String; uses MyApp.Very.Long.Namespace.One as One; uses MyApp.Very.Long.Namespace.Two as Two; begin Console.WriteLn('ok'); end program;"#,
        );
        assert!(formatted.contains("uses Std.Console as Console;\nuses Std.Conv as Conv;\n"));
        assert!(formatted.contains("MyApp.Very.Long.Namespace.Two"));
    }

    #[test]
    fn round_trip_hello() {
        let source = r#"program Hello;
uses Std.Console as Console;
begin
  Console.WriteLn('Hello, World!');
end program;
"#;
        let formatted = parse_and_format(source);
        let (_, errors) = parse_compilation_unit(&formatted);
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(
            formatted,
            format_program(&fpas_parser::parse(source).0, &CommentMap::default())
        );
    }

    #[test]
    fn unit_qualified_name() {
        let formatted = parse_and_format(
            r#"unit App.Math; function Scale(Value: integer): integer; begin return Value * 2; end function;
end unit;
"#,
        );
        assert!(formatted.starts_with(
            r#"unit App.Math;

"#
        ));
        assert!(formatted.contains("function Scale"));
    }
}
