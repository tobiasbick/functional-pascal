use super::*;

#[test]
fn build_program_rewrites_select_bindings_and_types() {
    let dir = create_temp_dir("link-select");
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
uses App.Types;
begin
  select
    case Value: Number from Ch:
      Use(Value)
  end
end.
",
    );
    write_text(
        &dir.join("src/types.fpas"),
        "\
unit App.Types;
type Number = integer;
const Value: Number := 42;
function Use(V: Number): Number;
begin
  return V
end;
",
    );

    let loaded = load_project(&project_file).expect("project should load");
    let program = build_program(
        loaded.main.as_deref().expect("main path must exist"),
        &loaded.source_files,
    )
    .expect("project should link");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    let Stmt::Select { arms, .. } = &program.body[0] else {
        panic!("expected select statement");
    };
    assert_eq!(arms[0].binding, "Value");
    support::assert_named_type(&arms[0].type_expr, &["App", "Types", "Number"]);
    let Expr::Designator(channel_designator) = &arms[0].channel else {
        panic!("expected channel designator");
    };
    support::assert_single_ident(channel_designator.parts.as_slice(), "Ch");
    let Stmt::Call {
        designator, args, ..
    } = &arms[0].body
    else {
        panic!("expected select body call");
    };
    support::assert_qualified_designator(designator.parts.as_slice(), &["App", "Types", "Use"]);
    let Expr::Designator(binding_designator) = &args[0] else {
        panic!("expected select binding argument");
    };
    support::assert_single_ident(binding_designator.parts.as_slice(), "Value");
}
