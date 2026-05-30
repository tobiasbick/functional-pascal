use super::super::support;
use crate::test_support::{create_temp_dir, write_text};
use std::fs;

#[test]
fn std_args_receives_program_arguments_after_cli_separator() {
    let cwd = create_temp_dir("std-args-cli");
    let path = cwd.join("main.fpas");
    write_text(
        &path,
        r#"program T;
uses Std.Console, Std.Args;
begin
  WriteLn(ParamCount());
  WriteLn(ParamStr(0));
  WriteLn(ParamStr(1))
end.
"#,
    );

    let args = vec![
        path.to_string_lossy().to_string(),
        String::from("--"),
        String::from("one"),
        String::from("-two"),
    ];
    let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\none\n-two\n");
}
