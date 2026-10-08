//! `program` and `unit` compilation units.

use fpas_parser::{Import, Program, Unit};

use crate::comments::{
    CommentMap, emit_leading_comments, emit_trailing_comments, emit_trailing_end_comments,
};

use super::Emitter;
use super::decl::emit_decls;
use super::stmt::emit_stmts_in_block;
use super::types::emit_qualified_id;
use super::wrap::{emit_wrapped_comma_list, measure_emit};

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
    if let Some(anchor) = comments.uses_anchor() {
        emit_leading_comments(emitter, comments, anchor, false);
    }
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
    emitter.write("end.");
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
    if let Some(anchor) = comments.uses_anchor() {
        emit_leading_comments(emitter, comments, anchor, false);
    }
    emit_optional_uses(emitter, &unit.uses, comments);
    if !unit.declarations.is_empty() {
        emit_decls(emitter, &unit.declarations, comments);
    }
    if let Some(anchor) = comments.closer_anchor(unit.span.offset) {
        emit_leading_comments(emitter, comments, anchor, false);
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
    if uses.iter().any(|unit_name| {
        let offset = unit_name.span.offset;
        !comments.leading_at(offset).is_empty() || !comments.trailing_at(offset).is_empty()
    }) {
        emit_commented_uses(emitter, uses, comments);
        emitter.blank_line();
        return;
    }
    let items: Vec<String> = uses
        .iter()
        .map(|unit_name| measure_emit(|inner| emit_import(inner, unit_name)))
        .collect();
    emit_wrapped_comma_list(emitter, "uses ", crate::style::INDENT_WIDTH, &items, ";");
    emitter.blank_line();
}

fn emit_commented_uses(emitter: &mut Emitter, uses: &[Import], comments: &CommentMap) {
    emitter.writeln("uses");
    emitter.with_indent(|inner| {
        for (index, unit_name) in uses.iter().enumerate() {
            let offset = unit_name.span.offset;
            emit_leading_comments(inner, comments, offset, false);
            inner.write_current_indent();
            emit_import(inner, unit_name);
            inner.write(if index + 1 == uses.len() { ";" } else { "," });
            emit_trailing_comments(inner, comments, offset);
            if !inner.ends_with_newline() {
                inner.write_line_end();
            }
        }
    });
}

fn emit_import(emitter: &mut Emitter, import: &Import) {
    emit_qualified_id(emitter, &import.unit);
    if let Some(alias) = &import.alias {
        emitter.write(" as ");
        emitter.write(&alias.name);
    }
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
        let formatted = parse_and_format("program Hello; begin WriteLn('Hello, World!'); end.");
        assert_eq!(
            formatted,
            "program Hello;\n\nbegin\n  WriteLn('Hello, World!');\nend.\n"
        );
    }

    #[test]
    fn program_with_uses() {
        let formatted = parse_and_format(
            "program Hello; uses Std.Console; begin WriteLn('Hello, World!'); end.",
        );
        assert_eq!(
            formatted,
            "program Hello;\n\nuses Std.Console;\n\nbegin\n  WriteLn('Hello, World!');\nend.\n"
        );
    }

    #[test]
    fn unit_clamp_emits_named_conditional_ending() {
        let source = "unit MyApp.Utils; uses Std.Math; function Clamp(Value: integer; Min: integer; Max: integer): integer; begin if Value < Min then return Min; elsif  Value > Max then return Max; else return Value; end if; end function; function IsBlank(S: string): boolean; begin return Length(Trim(S)) = 0; end function;\nend unit;";
        let formatted = parse_and_format(source);
        assert!(formatted.starts_with("unit MyApp.Utils;\n\nuses Std.Math;\n\n"));
        assert!(
            formatted.contains("if Value < Min then\n    return Min;\n  elsif Value > Max then")
        );
        assert!(formatted.contains("\n  end if;\nend function;"));
        assert!(formatted.contains("function IsBlank"));
    }

    #[test]
    fn program_type_then_begin() {
        let formatted = parse_and_format(
            "program T; type Point = record X: integer; Y: integer; end record; begin const P: Point := record X := 1; Y := 2; end; end.",
        );
        assert!(formatted.contains("type Point = record\n"));
        assert!(formatted.contains("end record;\n\nbegin\n"));
    }

    #[test]
    fn array_literal_short_stays_single_line() {
        let formatted = parse_and_format(
            "program T; begin const Words: array of string := ['red', 'green', 'blue']; end.",
        );
        assert!(
            formatted.contains("['red', 'green', 'blue']"),
            "formatted:\n{formatted}"
        );
    }

    #[test]
    fn long_uses_clause_wraps() {
        let formatted = parse_and_format(
            "program LongUses; uses Std.Console, Std.Conv, Std.String, MyApp.Very.Long.Namespace.One, MyApp.Very.Long.Namespace.Two; begin WriteLn('ok'); end.",
        );
        assert!(formatted.contains("uses\n"));
        assert!(formatted.contains("MyApp.Very.Long.Namespace.Two"));
    }

    #[test]
    fn import_aliases_preserve_spelling_comments_and_round_trip() {
        let source = "program T;\nuses\n  Std.Math AS Numbers, // calculation\n  App.As aS As; // contextual name\nbegin end.";
        let formatted = parse_and_format(source);
        assert!(formatted.contains("Std.Math as Numbers,"), "{formatted}");
        assert!(formatted.contains("App.As as As;"), "{formatted}");
        assert_eq!(formatted.matches("// calculation").count(), 1);
        assert_eq!(formatted.matches("// contextual name").count(), 1);
        assert_eq!(parse_and_format(&formatted), formatted);
        let long = parse_and_format(
            "unit Demo.Consumer; uses App.Very.Long.Namespace.One as FirstNamespace, App.Very.Long.Namespace.Two as SecondNamespace, Std.Math as Numbers; end unit;",
        );
        assert!(long.contains("uses\n"), "{long}");
        assert_eq!(parse_and_format(&long), long);
    }

    #[test]
    fn round_trip_hello() {
        let source =
            "program Hello;\nuses Std.Console;\nbegin\n  WriteLn('Hello, World!');\nend.\n";
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
            "unit App.Math; function Scale(Value: integer): integer; begin return Value * 2; end function;\nend unit;",
        );
        assert!(formatted.starts_with("unit App.Math;\n\n"));
        assert!(formatted.contains("function Scale"));
    }
}
