//! Task-aware editor signatures preserve the distinction between values and procedures.

use super::{LanguageService, TempDirectory};
use std::path::Path;

#[test]
fn task_wait_signature_uses_the_handle_result_type() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (routine, annotation, call, expected) in [
        (
            "function Work(): integer; begin return 42; end function;",
            "task of integer",
            "const Value: integer := Tasks.Wait(Job);",
            "function Wait(Handle: task of integer): integer",
        ),
        (
            "procedure Work(); begin null; end procedure;",
            "task",
            "Tasks.Wait(Job);",
            "procedure Wait(Handle: task)",
        ),
    ] {
        let temp = TempDirectory::new("task-signature");
        let source = format!(
            "program T; uses Std.Tasks as Tasks; {routine} begin scope  const Job: {annotation} := go Work(); {call}  end scope; end."
        );
        let path = temp.write("task.fpas", &source);
        let mut service =
            LanguageService::load_with_standard_library(temp.path(), &repository.join("lib"))
                .expect("standard library");
        let cursor = source.find("Wait(Job").expect("Wait call") + "Wait(Job".len();
        let help = service
            .signature_help(&path, cursor)
            .expect("task signature help")
            .value
            .expect("Wait signature");
        assert_eq!(help.signature.label, expected);
    }
}
