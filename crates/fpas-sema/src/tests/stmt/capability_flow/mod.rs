//! Flow-sensitive callable capability joins at assignments and control-flow edges.

mod assignments;
mod branches;
mod loops;
mod patterns;
mod repeat_returns;

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE;

fn program(types: &str, body: &str) -> String {
    format!(
        "program Flow; uses Std.Tasks as Tasks;
        {types}
        function Free(): integer; begin return 1; end function;
        procedure Main(Flag: boolean); begin scope
          var Count: integer := 0;
          const Local: function(): integer := function(): integer
            begin Count := Count + 1; return Count; end function;
          var Chosen: function(): integer := Free;
          {body}
        end scope; end procedure;
        begin Main(true); end."
    )
}

fn rejects_local(types: &str, body: &str) {
    let errors = check_errors(&program(types, body));
    assert!(
        !errors.is_empty(),
        "expected local callable rejection: {body}"
    );
    assert!(
        errors
            .iter()
            .all(|error| error.code == SEMA_TASK_BOUND_CALLABLE),
        "unexpected diagnostic for {body}: {errors:#?}"
    );
}
