//! `program` and `unit` compilation units.

use fpas_parser::{Program, QualifiedId, Unit};

use super::Emitter;
use super::decl::emit_decls;
use super::stmt::emit_stmts_in_block;
use super::types::emit_qualified_id;
use super::wrap::{emit_wrapped_comma_list, measure_emit};

/// Formats a `program` compilation unit.
#[must_use]
pub(crate) fn format_program(program: &Program) -> String {
    let mut emitter = Emitter::new();
    emit_program(&mut emitter, program);
    emitter.finish()
}

/// Formats a `unit` compilation unit.
#[must_use]
pub(crate) fn format_unit(unit: &Unit) -> String {
    let mut emitter = Emitter::new();
    emit_unit(&mut emitter, unit);
    emitter.finish()
}

fn emit_program(emitter: &mut Emitter, program: &Program) {
    emitter.writeln(&format!("program {};", program.name));
    emitter.blank_line();
    emit_optional_uses(emitter, &program.uses);
    if !program.declarations.is_empty() {
        emit_decls(emitter, &program.declarations);
        emitter.blank_line();
    }
    emitter.writeln("begin");
    emitter.with_indent(|inner| emit_stmts_in_block(inner, &program.body));
    emitter.writeln("end.");
}

fn emit_unit(emitter: &mut Emitter, unit: &Unit) {
    emitter.write("unit ");
    emit_qualified_id(emitter, &unit.name);
    emitter.write(";\n");
    emitter.blank_line();
    emit_optional_uses(emitter, &unit.uses);
    if !unit.declarations.is_empty() {
        emit_decls(emitter, &unit.declarations);
    }
}

fn emit_optional_uses(emitter: &mut Emitter, uses: &[QualifiedId]) {
    if uses.is_empty() {
        return;
    }
    let items: Vec<String> = uses
        .iter()
        .map(|unit_name| measure_emit(|inner| emit_qualified_id(inner, unit_name)))
        .collect();
    emit_wrapped_comma_list(emitter, "uses ", crate::style::INDENT_WIDTH, &items, ";");
    emitter.blank_line();
}

#[cfg(test)]
mod tests {
    use super::format_program;
    use crate::format_compilation_unit;
    use fpas_parser::parse_compilation_unit;

    fn parse_and_format(source: &str) -> String {
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        format_compilation_unit(&unit)
    }

    #[test]
    fn minimal_program() {
        let formatted = parse_and_format("program Hello; begin WriteLn('Hello, World!') end.");
        assert_eq!(
            formatted,
            "program Hello;\n\nbegin\n  WriteLn('Hello, World!')\nend.\n"
        );
    }

    #[test]
    fn program_with_uses() {
        let formatted = parse_and_format(
            "program Hello; uses Std.Console; begin WriteLn('Hello, World!') end.",
        );
        assert_eq!(
            formatted,
            "program Hello;\n\nuses Std.Console;\n\nbegin\n  WriteLn('Hello, World!')\nend.\n"
        );
    }

    #[test]
    fn unit_clamp_expands_branch_blocks() {
        let source = "unit MyApp.Utils; uses Std.Math; function Clamp(Value: integer; Min: integer; Max: integer): integer; begin if Value < Min then return Min else if Value > Max then return Max else return Value end; function IsBlank(S: string): boolean; begin return Length(Trim(S)) = 0 end;";
        let formatted = parse_and_format(source);
        assert!(formatted.starts_with("unit MyApp.Utils;\n\nuses Std.Math;\n\n"));
        assert!(formatted.contains("if Value < Min then\n  begin\n    return Min\n  end"));
        assert!(!formatted.contains("private function Hidden"));
        assert!(formatted.contains("function IsBlank"));
    }

    #[test]
    fn program_type_then_begin() {
        let formatted = parse_and_format(
            "program T; type Point = record X: integer; Y: integer; end; begin var P: Point := record X := 1; Y := 2; end end.",
        );
        assert!(formatted.contains("type\n  Point = record\n"));
        assert!(formatted.contains("end;\n\nbegin\n"));
    }

    #[test]
    fn array_literal_short_stays_single_line() {
        let formatted = parse_and_format(
            "program T; begin var Words: array of string := ['red', 'green', 'blue']; end.",
        );
        assert!(
            formatted.contains("['red', 'green', 'blue']"),
            "formatted:\n{formatted}"
        );
    }

    #[test]
    fn long_uses_clause_wraps() {
        let formatted = parse_and_format(
            "program LongUses; uses Std.Console, Std.Conv, Std.Array, Std.Dict, Std.Option, Std.Result, Std.String, MyApp.Very.Long.Namespace.One, MyApp.Very.Long.Namespace.Two; begin WriteLn('ok') end.",
        );
        assert!(formatted.contains("uses\n"));
        assert!(formatted.contains("MyApp.Very.Long.Namespace.Two"));
    }

    #[test]
    fn round_trip_hello() {
        let source = "program Hello;\nuses Std.Console;\nbegin\n  WriteLn('Hello, World!')\nend.\n";
        let formatted = parse_and_format(source);
        let (_, errors) = parse_compilation_unit(&formatted);
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(formatted, format_program(&fpas_parser::parse(source).0));
    }

    #[test]
    fn unit_qualified_name() {
        let formatted = parse_and_format(
            "unit App.Math; function Scale(Value: integer): integer; begin return Value * 2 end;",
        );
        assert!(formatted.starts_with("unit App.Math;\n\n"));
        assert!(formatted.contains("function Scale"));
    }
}
