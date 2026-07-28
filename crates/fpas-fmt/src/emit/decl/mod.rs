//! Declarations (const, var, type, routines).

mod group;
mod item;

use fpas_parser::Decl;

use crate::comments::CommentMap;

use super::Emitter;

pub(crate) use group::emit_decls;
pub(crate) use item::emit_decl;

/// Formats a declaration list (unit declarations or program type / top-level decls).
#[must_use]
pub(crate) fn format_decls(decls: &[Decl]) -> String {
    let mut emitter = Emitter::new();
    emit_decls(&mut emitter, decls, &CommentMap::default());
    emitter.finish()
}

#[cfg(test)]
mod tests {
    use super::format_decls;
    use fpas_parser::parse_compilation_unit;

    fn format_unit_decls(source: &str) -> String {
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        let fpas_parser::CompilationUnit::Unit(unit) = unit else {
            panic!("expected unit");
        };
        format_decls(&unit.declarations)
    }

    fn format_program_decls(source: &str) -> String {
        let (program, errors) = fpas_parser::parse(source);
        assert!(errors.is_empty(), "{errors:?}");
        format_decls(&program.declarations)
    }

    #[test]
    fn record_one_field() {
        let formatted =
            format_program_decls("program T; type IdBox = record Value: integer; end; begin end.");
        assert_eq!(
            formatted,
            "type\n  IdBox = record\n    Value: integer;\n  end;\n"
        );
    }

    #[test]
    fn record_five_fields() {
        let formatted = format_program_decls(
            "program T; type Person = record Id: integer; Name: string; Age: integer; Active: boolean; Score: real; end; begin end.",
        );
        assert!(formatted.contains("Person = record\n"));
        assert!(formatted.contains("Id: integer;\n"));
        assert!(formatted.contains("Score: real;\n"));
    }

    #[test]
    fn record_with_defaults_and_methods() {
        let formatted = format_program_decls(
            "program T;
type
  Point = record
    X: integer;
    Y: integer;
    function Sum(Self: Point): integer;
    begin
      return Self.X + Self.Y
    end;
  end;
begin
end.",
        );
        assert!(
            formatted.contains("X: integer;\n    Y: integer;\n\n    function Sum"),
            "formatted:\n{formatted}"
        );
        assert!(formatted.contains("return Self.X + Self.Y"));
        assert!(formatted.contains("end;\n  end;\n"));
    }

    #[test]
    fn enum_and_alias() {
        let formatted = format_program_decls(
            "program T; type Color = enum Red; Green; Blue; end; IntAlias = integer; begin end.",
        );
        assert!(formatted.contains("Color = enum\n    Red;\n    Green;\n    Blue;\n  end;\n"));
        assert!(formatted.contains("IntAlias = integer;\n"));
    }

    #[test]
    fn unit_function_visibility() {
        let formatted = format_unit_decls(
            "unit MyApp.Utils; public function Clamp(Value: integer; Min: integer; Max: integer): integer; begin if Value < Min then begin return Min end else begin return Value end end; function Hidden(): integer; begin return 0 end;",
        );
        assert!(formatted.contains("public function Clamp"));
        assert!(formatted.contains("\nfunction Hidden"));
    }

    #[test]
    fn unit_default_private_vars_and_consts_are_block_grouped() {
        let formatted = format_unit_decls(
            "unit U; mutable var A: integer := 1; mutable var B: integer := 2; const C: integer := 3; const D: integer := 4;",
        );
        assert!(formatted.contains("mutable var\n  A: integer := 1;\n  B: integer := 2;\n"));
        assert!(formatted.contains("const\n  C: integer := 3;\n  D: integer := 4;\n"));
    }

    #[test]
    fn unit_default_private_type_uses_type_block() {
        let formatted = format_unit_decls("unit U; type Complex = record Re: real; Im: real; end;");
        assert!(
            formatted.contains("type\n  Complex = record\n"),
            "formatted:\n{formatted}"
        );
    }
}
