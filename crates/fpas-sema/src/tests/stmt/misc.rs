use super::super::{check_errors, check_ok};

#[test]
fn panic_with_string() {
    check_ok("program T; begin panic('error') end.");
}

#[test]
fn panic_with_integer_error() {
    check_errors("program T; begin panic(42) end.");
}

#[test]
fn inline_var() {
    check_ok("program T; begin var X: integer := 42 end.");
}

#[test]
fn inline_mutable_var() {
    check_ok(
        "program T; begin \
         mutable var X: integer := 0; \
         X := 1 \
         end.",
    );
}
