use super::super::{check_errors, check_ok};

#[test]
fn panic_with_string() {
    check_ok(r#"program T; begin panic('error'); end program;"#);
}

#[test]
fn panic_with_integer_error() {
    check_errors(r#"program T; begin panic(42); end program;"#);
}

#[test]
fn inline_var() {
    check_ok(r#"program T; begin var X: integer := 42; end program;"#);
}

#[test]
fn inline_mutable_var() {
    check_ok(r#"program T; begin mutable var X: integer := 0; X := 1; end program;"#);
}
