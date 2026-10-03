use super::super::{check_errors, check_ok};

#[test]
fn break_inside_loop() {
    check_ok(r#"program T; begin while true do break; end while; end program;"#);
}

#[test]
fn break_outside_loop() {
    check_errors(r#"program T; begin break; end program;"#);
}

#[test]
fn continue_inside_loop() {
    check_ok(r#"program T; begin while true do continue; end while; end program;"#);
}

#[test]
fn continue_outside_loop() {
    check_errors(r#"program T; begin continue; end program;"#);
}

#[test]
fn break_in_nested_loop() {
    check_ok(
        r#"program T; begin while true do while true do break; end while; end while; end program;"#,
    );
}

#[test]
fn break_in_if_outside_loop() {
    check_errors(r#"program T; begin if true then break; end if; end program;"#);
}

#[test]
fn continue_in_if_outside_loop() {
    check_errors(r#"program T; begin if true then continue; end if; end program;"#);
}

#[test]
fn break_in_function_body_not_in_loop() {
    check_errors(
        r#"program T; function Foo(): integer; begin break; return 0; end function; begin discard Foo(); end program;"#,
    );
}

#[test]
fn continue_in_function_body_not_in_loop() {
    check_errors(
        r#"program T; procedure Bar(); begin continue; end procedure; begin Bar(); end program;"#,
    );
}

#[test]
fn break_inside_for_loop() {
    check_ok(r#"program T; begin for I: integer := 1 to 5 do break; end for; end program;"#);
}

#[test]
fn continue_inside_for_loop() {
    check_ok(r#"program T; begin for I: integer := 1 to 5 do continue; end for; end program;"#);
}

#[test]
fn break_inside_repeat_loop() {
    check_ok(r#"program T; begin repeat break; until true; end program;"#);
}

#[test]
fn continue_inside_repeat_loop() {
    check_ok(r#"program T; begin repeat continue; until true; end program;"#);
}

#[test]
fn break_in_nested_if_inside_loop() {
    check_ok(
        r#"program T; begin while true do if true then if true then break; end if; end if; end while; end program;"#,
    );
}

#[test]
fn continue_in_nested_if_inside_loop() {
    check_ok(
        r#"program T; begin for I: integer := 1 to 5 do if true then if true then continue; end if; end if; end for; end program;"#,
    );
}
