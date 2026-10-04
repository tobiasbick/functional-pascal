//! Compilation and execution of the handbook's actual routine and branch examples.

use super::{cli, example, fs, write_text};

#[test]
fn documented_case_and_callbacks_execute_before_and_after_cli_formatting() {
    for (path, index, expected) in [
        (
            "docs/pascal/language/pattern-matching/syntax.md",
            0,
            "retry\n",
        ),
        ("docs/pascal/language/functions/closures.md", 1, "42\n"),
    ] {
        let cwd = super::create_temp_dir("handbook-execution");
        write_text(&cwd.join("main.fpas"), &example(path, index));
        for pass in 0..2 {
            let (code, stdout, stderr) = cli(&cwd, &["run", "main.fpas"]);
            assert_eq!(code, 0, "{path}, pass {pass}: {stderr}");
            assert_eq!(stdout, expected, "{path}, pass {pass}");
            let (code, _, stderr) = cli(&cwd, &["fmt", "main.fpas"]);
            assert_eq!(code, 0, "{path}: {stderr}");
        }
        let (code, _, stderr) = cli(&cwd, &["fmt", "--check", "main.fpas"]);
        fs::remove_dir_all(&cwd).expect("remove test fixture");
        assert_eq!(code, 0, "{stderr}");
    }
}

#[test]
fn documented_routine_imports_compile_and_handle_empty_and_boundary_inputs() {
    for (path, index, body, expected) in [
        (
            "docs/pascal/language/functions/early-return.md",
            0,
            "Console.WriteLn(IndexOf([], 'x')); Console.WriteLn(IndexOf(['a', 'b'], 'b')); \
             Console.WriteLn(IndexOf(['a'], 'x'));",
            "-1\n1\n-1\n",
        ),
        (
            "docs/pascal/language/functions/nested.md",
            0,
            "Console.WriteLn(Hypotenuse(3.0, 4.0)); Console.WriteLn(Hypotenuse(0.0, 0.0));",
            "5\n0\n",
        ),
        (
            "docs/pascal/language/error-handling/option.md",
            1,
            "case FindIndex([], 1) of when Option.Some(const I): panic('unexpected match'); when Option.None: Console.WriteLn('empty'); end case; case FindIndex([10, 20], 20) of when Option.Some(const I): Console.WriteLn(I); when Option.None: panic('missing match'); end case;",
            "empty\n1\n",
        ),
    ] {
        let declarations = example(path, index);
        let source = format!(
            "program DocumentedRoutines;\nuses Std.Console as Console;\n\
             {declarations}\nbegin\n{body}\nend program;"
        );
        let cwd = super::create_temp_dir("handbook-routines");
        write_text(&cwd.join("main.fpas"), &source);
        let (code, stdout, stderr) = cli(&cwd, &["run", "main.fpas"]);
        fs::remove_dir_all(&cwd).expect("remove test fixture");
        assert_eq!(code, 0, "{path}: {stderr}");
        assert_eq!(stdout, expected, "{path}");
    }
}

#[test]
fn documented_task_waits_use_their_import_alias_and_preserve_result_types() {
    let path = "docs/pascal/language/concurrency/task-handles.md";
    let routines = "const Data: integer := 7; function Compute(Value: integer): integer; begin return Value; end function; function ComputeSomething(Value: integer): integer; begin return Value; end function; function Connect(): result of (boolean, string); begin return Result.Ok(true); end function; function Serve(): result of (boolean, string); begin return Result.Ok(true); end function;";
    for (index, declarations, before, after, expected) in [
        (0, false, "", "Console.WriteLn(Tasks.Wait(T));", "7\n"),
        (
            1,
            true,
            "",
            "Tasks.WaitAll(Jobs); Console.WriteLn(Doubled(go Compute(21)));",
            "42\n",
        ),
        (2, false, "", "Console.WriteLn(TaskValue);", "100\n"),
        (
            3,
            false,
            "var T1: task := go Compute(1); var T2: task := go Compute(2); var T3: task := go Compute(3);",
            "Console.WriteLn(Tasks.Wait(T2));",
            "2\n",
        ),
    ] {
        let source = example(path, index);
        let fragment = if index == 0 {
            source.as_str()
        } else {
            source
                .strip_prefix("uses Std.Tasks as Tasks;\n\n")
                .expect("documented wait example must declare its alias")
        };
        let (declarations, body) = if declarations {
            (fragment, "")
        } else {
            ("", fragment)
        };
        let source = format!(
            "program DocumentedTasks; uses Std.Console as Console; uses Std.Tasks as Tasks; \
             {routines} {declarations} begin {before} {body} {after} end program;"
        );
        let cwd = super::create_temp_dir("handbook-task-waits");
        write_text(&cwd.join("main.fpas"), &source);
        let (code, stdout, stderr) = cli(&cwd, &["run", "main.fpas"]);
        fs::remove_dir_all(&cwd).expect("remove test fixture");
        assert_eq!(code, 0, "{path}, example {index}: {stderr}");
        assert_eq!(stdout, expected, "{path}, example {index}");
    }
}
