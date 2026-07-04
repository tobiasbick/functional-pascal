use crate::cli_input::TestCliConfig;
use crate::cli_test::test_cli;
use crate::test_support::{create_temp_dir, write_text};

#[test]
fn test_cli_rejects_unit_file_as_test_entry() {
    let cwd = create_temp_dir("fpas-test-unit-reject");
    write_text(
        &cwd.join("helper_test.fpas"),
        "unit Tests.Helper;\nprocedure X();\nbegin end;",
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.join("helper_test.fpas")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: None,
            jobs: 1,
            strict: false,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 2);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("must be `program` files"), "stderr={text}");
    assert!(text.contains("unit Tests.Helper"), "stderr={text}");
}
