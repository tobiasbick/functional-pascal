use super::*;

#[test]
fn build_program_rewrites_lambda_signatures_and_body() {
    let dir = create_temp_dir("link-lambda");
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
  var F: integer := function(X: Number): Number
  begin
    return Identity(X)
  end
end.
",
    );
    write_text(
        &dir.join("src/types.fpas"),
        "\
unit App.Types;
type Number = integer;
function Identity(X: Number): Number;
begin
  return X
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

    let Stmt::Var(var_def) = &program.body[0] else {
        panic!("expected inline var");
    };
    let Expr::Function {
        params,
        return_type,
        body,
        ..
    } = &var_def.value
    else {
        panic!("expected lambda expression");
    };
    support::assert_named_type(&params[0].type_expr, &["App", "Types", "Number"]);
    support::assert_named_type(return_type, &["App", "Types", "Number"]);

    let fpas_parser::FuncBody::Block { stmts, .. } = body else {
        panic!("expected lambda block body");
    };
    let Stmt::Return(
        Some(Expr::Call {
            designator, args, ..
        }),
        _,
    ) = &stmts[0]
    else {
        panic!("expected lambda return call");
    };
    support::assert_qualified_designator(
        designator.parts.as_slice(),
        &["App", "Types", "Identity"],
    );
    let Expr::Designator(param_designator) = &args[0] else {
        panic!("expected lambda parameter argument");
    };
    support::assert_single_ident(param_designator.parts.as_slice(), "X");
}
