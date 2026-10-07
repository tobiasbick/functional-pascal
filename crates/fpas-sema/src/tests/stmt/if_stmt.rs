use super::super::{check_errors, check_ok};

#[test]
fn if_boolean_condition() {
    check_ok("program T; begin if true then return; end if; end.");
}

#[test]
fn if_non_boolean_condition() {
    check_errors("program T; begin if 42 then return; end if; end.");
}

#[test]
fn if_string_condition() {
    check_errors("program T; begin if 'hello' then return; end if; end.");
}

#[test]
fn if_else_ok() {
    check_ok("program T; begin if true then return; else return; end if; end.");
}

#[test]
fn if_else_non_boolean_condition() {
    check_errors("program T; begin if 42 then return; else return; end if; end.");
}

#[test]
fn if_comparison_condition() {
    check_ok(
        "program T; \
         const X: integer := 5; \
         begin if X > 0 then return; end if; end.",
    );
}

#[test]
fn if_else_if_chain_ok() {
    check_ok(
        "program T; \
         const X: integer := 5; \
         begin \
           if X > 10 then return; \
           elsif  X > 0 then return; \
           else return; \
         end if; end.",
    );
}

#[test]
fn if_with_block_ok() {
    check_ok(
        "program T; \
         var X: integer := 5; \
         begin \
           if X > 0 then begin X := 1; end; \
           else begin X := 2; end; \
         end if; end.",
    );
}
