//! Type-name conversion arguments fail semantically before compiler lowering.

use super::*;

#[test]
fn type_name_conversion_arguments_report_semantic_diagnostics() {
    let root = temp_dir();
    for declaration in [
        "type ValueType = 0..10;",
        "type ValueType = distinct integer;",
    ] {
        for argument in [
            "ValueType",
            "if true then ValueType else ValueType(1) end if",
        ] {
            write(
                &root.join("main.fpas"),
                &format!(
                    "program T; {declaration} begin const Value := ValueType({argument}); end."
                ),
            );
            for command in ["check", "run"] {
                let output = command_output(&root, command, "main.fpas", true);
                assert_eq!(output.status.code(), Some(1), "{output:?}");
                let records = stderr_records(&output);
                assert_eq!(records.len(), 1, "{records:#?}");
                assert_eq!(records[0]["code"], "FP3006");
                assert_eq!(records[0]["phase"], "sema");
                assert!(
                    records[0]["hint"]
                        .as_str()
                        .is_some_and(|hint| hint.contains("ValueType(Value)")),
                    "{records:#?}"
                );
            }
        }
    }
    fs::remove_dir_all(root).expect("remove fixtures");
}
