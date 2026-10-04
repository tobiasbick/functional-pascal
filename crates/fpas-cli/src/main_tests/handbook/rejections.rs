//! Invalid derivatives of actual handbook examples must retain migration guidance.

use super::{cli, example, fs, write_text};

#[test]
fn invalid_documented_block_forms_are_rejected_without_formatting_the_file() {
    let case = example("docs/pascal/language/pattern-matching/syntax.md", 0);
    let callback = example("docs/pascal/language/functions/closures.md", 1);
    for (source, hint) in [
        (case.replace("end case;", "end while;"), "end case"),
        (
            case.replace("Console.WriteLn(Message);", "Console.WriteLn(Message)"),
            ";",
        ),
        (case.replace("null;", ";"), "null;"),
        (case.replace("null;", ""), "null;"),
        (case.replace("end program;", "end."), "end program;"),
        (case.replace(" as Console", ""), "as"),
        (callback.replace("end function,", "end function;,"), ","),
        (callback.replace("end procedure);", "end procedure;);"), ")"),
    ] {
        let cwd = super::create_temp_dir("handbook-rejection");
        let path = cwd.join("invalid.fpas");
        write_text(&path, &source);
        let (code, stdout, stderr) = cli(&cwd, &["fmt", "invalid.fpas"]);
        let unchanged = fs::read_to_string(&path).expect("rejected source remains");
        fs::remove_dir_all(&cwd).expect("remove test fixture");
        assert_eq!(code, 1, "invalid example was formatted: {source}");
        assert!(stdout.is_empty(), "{stdout}");
        assert!(stderr.contains(hint), "missing {hint:?}: {stderr}");
        assert_eq!(unchanged, source);
    }
}

#[test]
fn documented_alias_and_case_scope_rules_reject_unqualified_or_escaping_names() {
    let case = example("docs/pascal/language/pattern-matching/syntax.md", 0);
    for (source, code, message) in [
        (
            case.replace("Console.WriteLn(Message)", "WriteLn(Message)"),
            "F2003",
            "Unknown procedure `WriteLn`",
        ),
        (
            case.replace(
                "const Status: integer := 1;",
                "const console: integer := 1;",
            ),
            "F2002",
            "Declaration `console` conflicts with an import alias",
        ),
        (
            case.replace("end case;", "end case; Console.WriteLn(Message);"),
            "F2003",
            "Undefined identifier `Message`",
        ),
    ] {
        let cwd = super::create_temp_dir("handbook-name-rejection");
        let path = cwd.join("invalid.fpas");
        write_text(&path, &source);
        let (exit, _, stderr) = cli(&cwd, &["check", "invalid.fpas"]);
        let unchanged = fs::read_to_string(&path).expect("checked source remains");
        fs::remove_dir_all(&cwd).expect("remove test fixture");
        assert_eq!(exit, 1, "invalid names compiled: {source}");
        assert!(
            stderr.contains(code) && stderr.contains(message),
            "{stderr}"
        );
        assert_eq!(unchanged, source);
    }
}
