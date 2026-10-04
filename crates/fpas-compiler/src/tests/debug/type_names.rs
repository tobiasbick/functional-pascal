//! Canonical generic names in portable debugger metadata.

use super::parse_ok;

#[test]
fn nested_debug_type_names_parse_as_source_types() {
    let program = parse_ok(
        r#"program DebugTypeNames;
procedure Inspect(
  Data: result of (array of (dict of (string, option of (integer))), string);
  Child: task of (integer);
  Messages: channel of (option of (integer)));
begin
  null;
end procedure;
begin
  null;
end program;"#,
    );
    let executable = crate::compile(&program).expect("typed debug parameters");
    let image = executable.executable();
    let function = image
        .functions
        .iter()
        .find(|function| image.strings.get(function.name) == Some("inspect"))
        .expect("Inspect metadata");
    for (name, expected) in [
        (
            "Data",
            "result of (array of (dict of (string, option of (integer))), string)",
        ),
        ("Child", "task of (integer)"),
        ("Messages", "channel of (option of (integer))"),
    ] {
        let binding = function
            .debug
            .bindings
            .iter()
            .find(|binding| image.strings.get(binding.name) == Some(name))
            .expect("parameter binding");
        let rendered = image.strings.get(binding.type_name).expect("type name");
        assert_eq!(rendered, expected);
        parse_ok(&format!(
            "program TypeName; type Displayed = {rendered}; begin null; end program;"
        ));
    }
}
