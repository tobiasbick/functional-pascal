use crate::cli_input::TestCliConfig;
use crate::cli_test::test_cli;
use crate::test_support::{create_temp_dir, write_text};
use std::time::Duration;

#[test]
fn test_cli_timeout_aborts_infinite_loop() {
    let cwd = create_temp_dir("fpas-test-timeout");
    write_text(
        &cwd.join("hang_test.fpas"),
        "program H;\nbegin\n  while 1 = 1 do\n  begin\n  end\nend.",
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.join("hang_test.fpas")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: Some(Duration::from_secs(1)),
            jobs: 1,
            strict: false,
            standard_library: None,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 3);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("TIMEOUT  hang_test.fpas"));
}
