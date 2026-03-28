use super::super::{check_errors, check_ok};

#[test]
fn while_boolean_condition() {
    check_ok("program T; begin while true do return end.");
}

#[test]
fn while_non_boolean_condition() {
    check_errors("program T; begin while 42 do return end.");
}

#[test]
fn while_string_condition() {
    check_errors("program T; begin while 'yes' do return end.");
}

#[test]
fn while_comparison_condition() {
    check_ok(
        "program T; \
         var X: integer := 5; \
         begin while X > 0 do return end.",
    );
}

#[test]
fn while_complex_boolean_condition() {
    check_ok(
        "program T; \
         var X: integer := 5; \
         begin while (X > 0) and (X < 10) do return end.",
    );
}

#[test]
fn while_real_condition() {
    check_errors("program T; begin while 3.14 do return end.");
}

#[test]
fn while_char_condition() {
    check_errors("program T; begin while 'A' do return end.");
}

#[test]
fn while_false_literal() {
    check_ok("program T; begin while false do return end.");
}

#[test]
fn while_not_expression_condition() {
    check_ok(
        "program T; \
         var Done: boolean := false; \
         begin while not Done do return end.",
    );
}

#[test]
fn repeat_boolean_condition() {
    check_ok("program T; begin repeat return until true end.");
}

#[test]
fn repeat_non_boolean_condition() {
    check_errors("program T; begin repeat return until 42 end.");
}

#[test]
fn repeat_string_condition() {
    check_errors("program T; begin repeat return until 'yes' end.");
}

#[test]
fn repeat_real_condition() {
    check_errors("program T; begin repeat return until 3.14 end.");
}

#[test]
fn repeat_comparison_condition() {
    check_ok(
        "program T; \
         mutable var X: integer := 0; \
         begin repeat X := X + 1 until X > 5 end.",
    );
}

#[test]
fn repeat_complex_boolean_condition() {
    check_ok(
        "program T; \
         mutable var X: integer := 0; \
         begin repeat X := X + 1 until (X > 0) and (X < 10) end.",
    );
}

#[test]
fn repeat_false_literal() {
    check_ok("program T; begin repeat break until false end.");
}

#[test]
fn repeat_not_expression_condition() {
    check_ok(
        "program T; \
         mutable var Done: boolean := false; \
         begin repeat Done := true until not Done end.",
    );
}

#[test]
fn repeat_condition_cannot_use_body_local() {
    let errs = check_errors(
        "program T; begin \
         repeat \
           var X: integer := 1 \
         until X = 1 \
         end.",
    );
    assert!(
        errs.iter()
            .any(|err| err.message.contains("Undefined identifier `X`")),
        "expected unknown-name error for repeat body local in until condition, got {errs:#?}"
    );
}
