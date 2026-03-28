use super::super::{check_errors, check_ok};

#[test]
fn break_inside_loop() {
    check_ok(
        "program T; begin \
         while true do break \
         end.",
    );
}

#[test]
fn break_outside_loop() {
    check_errors("program T; begin break end.");
}

#[test]
fn continue_inside_loop() {
    check_ok(
        "program T; begin \
         while true do continue \
         end.",
    );
}

#[test]
fn continue_outside_loop() {
    check_errors("program T; begin continue end.");
}

#[test]
fn break_in_nested_loop() {
    check_ok(
        "program T; begin \
         while true do \
           while true do break \
         end.",
    );
}

#[test]
fn break_in_if_outside_loop() {
    check_errors(
        "program T; begin \
         if true then break \
         end.",
    );
}

#[test]
fn continue_in_if_outside_loop() {
    check_errors(
        "program T; begin \
         if true then continue \
         end.",
    );
}

#[test]
fn break_in_function_body_not_in_loop() {
    check_errors(
        "program T; \
         function Foo(): integer; \
         begin break; return 0 end; \
         begin Foo() end.",
    );
}

#[test]
fn continue_in_function_body_not_in_loop() {
    check_errors(
        "program T; \
         procedure Bar(); \
         begin continue end; \
         begin Bar() end.",
    );
}

#[test]
fn break_inside_for_loop() {
    check_ok(
        "program T; begin \
         for I: integer := 1 to 5 do break \
         end.",
    );
}

#[test]
fn continue_inside_for_loop() {
    check_ok(
        "program T; begin \
         for I: integer := 1 to 5 do continue \
         end.",
    );
}

#[test]
fn break_inside_repeat_loop() {
    check_ok(
        "program T; begin \
         repeat break until true \
         end.",
    );
}

#[test]
fn continue_inside_repeat_loop() {
    check_ok(
        "program T; begin \
         repeat continue until true \
         end.",
    );
}

#[test]
fn break_in_nested_if_inside_loop() {
    check_ok(
        "program T; begin \
         while true do \
           if true then \
             if true then break \
         end.",
    );
}

#[test]
fn continue_in_nested_if_inside_loop() {
    check_ok(
        "program T; begin \
         for I: integer := 1 to 5 do \
           if true then \
             if true then continue \
         end.",
    );
}
