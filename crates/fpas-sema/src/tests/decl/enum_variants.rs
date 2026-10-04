//! Enum variants require their declaring type even when a short name is unique.

use super::{check_errors, check_ok};

#[test]
fn unique_enum_variants_do_not_create_short_names() {
    for expression in ["Empty", "Full(42)"] {
        let errors = check_errors(&format!(
            "program Variants; type Choice = enum Empty; Full(Value: integer); end enum; \
             begin var Value: Choice := {expression}; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| { error.code == fpas_diagnostics::codes::SEMA_UNKNOWN_NAME }),
            "{expression}: {errors:?}"
        );
    }
}

#[test]
fn qualified_variant_and_same_named_routine_resolve_independently() {
    check_ok(
        r#"program Variants;
type Choice = enum Empty; Full(Value: integer); end enum;
function Full(Value: integer): integer;
begin
  return Value + 1;
end function;
begin
  var Selected: Choice := Choice.Full(42);
  var Number: integer := Full(42);
  case Selected of
    when Choice.Empty: null;
    when Choice.Full(const Value):
      if Value + 1 <> Number then panic('wrong resolution'); end if;
  end case;
end program;"#,
    );
}
