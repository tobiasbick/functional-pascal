use super::support::load_and_build_program;
use super::*;

#[test]
fn build_program_rewrites_case_patterns_guards_and_bindings() {
    let dir = create_temp_dir("link-case-pattern");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
    write_text(
        &dir.join("src/main.fpas"),
        "\
program Main;
uses App.Shape;
begin
  case Input of
    Shape.Rect(Value) if Check(Value):
      Use(Value)
  end
end.
",
    );
    write_text(
        &dir.join("src/shape.fpas"),
        "\
unit App.Shape;
type Shape = enum
  Rect(Value: integer);
end;
const Value: integer := 99;
function Check(V: integer): boolean;
begin
  return V > 0
end;
function Use(V: integer): integer;
begin
  return V
end;
",
    );

    let program = load_and_build_program(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    let Stmt::Case { arms, .. } = &program.body[0] else {
        panic!("expected case statement");
    };
    let CaseLabel::Value { start, .. } = &arms[0].labels[0] else {
        panic!("expected enum-pattern label");
    };
    let Expr::Call {
        designator, args, ..
    } = start
    else {
        panic!("expected enum-pattern call");
    };
    support::assert_qualified_designator(
        designator.parts.as_slice(),
        &["App", "Shape", "Shape", "Rect"],
    );
    let Expr::Designator(binding_designator) = &args[0] else {
        panic!("expected binding designator");
    };
    support::assert_single_ident(binding_designator.parts.as_slice(), "Value");

    let guard = arms[0].guard.as_ref().expect("guard must exist");
    let Expr::Call {
        designator, args, ..
    } = guard
    else {
        panic!("expected call in guard");
    };
    support::assert_qualified_designator(designator.parts.as_slice(), &["App", "Shape", "Check"]);
    let Expr::Designator(binding_designator) = &args[0] else {
        panic!("expected guard binding argument");
    };
    support::assert_single_ident(binding_designator.parts.as_slice(), "Value");

    let Stmt::Call {
        designator, args, ..
    } = &arms[0].body
    else {
        panic!("expected call body");
    };
    support::assert_qualified_designator(designator.parts.as_slice(), &["App", "Shape", "Use"]);
    let Expr::Designator(binding_designator) = &args[0] else {
        panic!("expected body binding argument");
    };
    support::assert_single_ident(binding_designator.parts.as_slice(), "Value");
}
