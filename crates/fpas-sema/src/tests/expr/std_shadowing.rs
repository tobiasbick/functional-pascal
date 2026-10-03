use crate::analyze_with_types;

#[test]
fn shadowed_standard_names_do_not_produce_intrinsic_call_metadata() {
    for routine in [
        "function Send(A: integer; B: integer; C: integer): integer; begin return A + B + C; end function;",
        "procedure Send(A: integer; B: integer; C: integer); begin null; end procedure;",
    ] {
        let consumer = if routine.starts_with("function") {
            "discard "
        } else {
            ""
        };
        let source = format!(
            "program Shadow; uses Std.Tasks as Tasks; uses Std.Console as Console; {routine} begin {consumer}Send(1, 2, 3); Console.WriteLn('ok'); {consumer}sEnD(1, 2, 3); end program;"
        );
        let (program, errors) = fpas_parser::parse(&source);
        assert!(errors.is_empty(), "{errors:?}");
        let metadata = analyze_with_types(&program);
        assert!(metadata.errors.is_empty(), "{:?}", metadata.errors);
        assert_eq!(
            metadata.intrinsic_calls.values().collect::<Vec<_>>(),
            vec!["Std.Console.WriteLn"],
        );
    }
}
