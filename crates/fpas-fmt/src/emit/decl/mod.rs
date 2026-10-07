//! Declarations (const, var, type, routines).

mod item;
mod list;
mod routines;

use fpas_parser::Decl;

use crate::comments::CommentMap;

use super::Emitter;

pub(crate) use item::emit_decl;
pub(crate) use list::emit_decls;

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
        let formatted = format_program_decls(
            "program T; type IdBox = record Value: integer; end record; begin end.",
        );
        assert_eq!(
            formatted,
            "type IdBox = record\n  Value: integer;\nend record;\n"
        );
    }

    #[test]
    fn record_five_fields() {
        let formatted = format_program_decls(
            "program T; type Person = record Id: integer; Name: string; Age: integer; Active: boolean; Score: real; end record; begin end.",
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
      return Self.X + Self.Y;
    end function;
  end record;
begin
end.",
        );
        assert!(
            formatted.contains("X: integer;\n  Y: integer;\n\n  function Sum"),
            "formatted:\n{formatted}"
        );
        assert!(formatted.contains("return Self.X + Self.Y;"));
        assert!(formatted.contains("end function;\nend record;\n"));
    }

    #[test]
    fn enum_and_alias() {
        let formatted = format_program_decls(
            "program T; type Color = enum Red; Green; Blue; end enum; type IntAlias = integer; begin end.",
        );
        assert!(formatted.contains("Color = enum\n  Red;\n  Green;\n  Blue;\nend enum;\n"));
        assert!(formatted.contains("IntAlias = integer;\n"));
    }

    #[test]
    fn unit_function_visibility() {
        let formatted = format_unit_decls(
            "unit MyApp.Utils; public function Clamp(Value: integer; Min: integer; Max: integer): integer; begin if Value < Min then begin return Min; end; else begin return Value; end; end if; end function; function Hidden(): integer; begin return 0; end function;\nend unit;",
        );
        assert!(formatted.contains("public function Clamp"));
        assert!(formatted.contains("\nfunction Hidden"));
    }

    #[test]
    fn unit_default_private_bindings_have_individual_keywords() {
        let formatted = format_unit_decls(
            "unit U; var A: integer := 1; var B: integer := 2; const C: integer := 3; const D: integer := 4;\nend unit;",
        );
        assert!(formatted.contains("var A: integer := 1;\nvar B: integer := 2;\n"));
        assert!(formatted.contains("const C: integer := 3;\nconst D: integer := 4;\n"));
    }

    #[test]
    fn unit_default_private_type_has_its_own_keyword() {
        let formatted = format_unit_decls(
            "unit U; type Complex = record Re: real; Im: real; end record;\nend unit;",
        );
        assert!(
            formatted.contains("type Complex = record\n"),
            "formatted:\n{formatted}"
        );
    }
}
