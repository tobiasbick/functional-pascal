//! Repeat conditions resolve names outside the body-local scope.

use super::super::assert_succeeds;

#[test]
fn repeat_condition_uses_the_outer_binding_after_fallthrough_and_continue() {
    for tail in ["", "; continue"] {
        assert_succeeds(&format!(
            "program RepeatScope; begin\n              var Done: boolean := true;\n              mutable var Count: integer := 0;\n              repeat\n                Count := Count + 1;\n                if Count > 1 then panic('until used a body-local binding'); end if;\n                var Done: boolean := false{tail};\n              until Done;\n              if not Done then panic('outer binding was changed'); end if;\n              if Count <> 1 then panic('repeat did not execute exactly once'); end if;\n            end program;"
        ));
    }
}

#[test]
fn repeat_body_shadow_can_have_a_different_type_from_the_condition() {
    assert_succeeds(
        r#"program RepeatTypes; begin
          var Done: boolean := true;
          repeat
            var Done: integer := 7;
            if Done <> 7 then panic('body did not use its own local'); end if;
          until Done;
        end program;"#,
    );
}
