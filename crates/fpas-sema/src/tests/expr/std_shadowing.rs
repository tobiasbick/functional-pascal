use crate::analyze_with_types;

#[test]
fn shadowed_standard_names_do_not_produce_intrinsic_call_metadata() {
    for routine in [
        "function Send(A: integer; B: integer; C: integer): integer; begin return A + B + C end;",
        "procedure Send(A: integer; B: integer; C: integer); begin end;",
    ] {
        let source = format!(
            "program Shadow; uses Std.Task; {routine} begin Send(1, 2, 3); Std.Console.WriteLn('ok'); sEnD(1, 2, 3) end."
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
