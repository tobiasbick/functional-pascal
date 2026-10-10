//! Distinct `case` selectors, typed labels, ranges, and nested pattern values.
//!
//! Documentation: `docs/pascal/language/types/distinct-types.md`

use fpas_diagnostics::codes::{SEMA_NON_CONSTANT_EXPRESSION, SEMA_TYPE_MISMATCH};

use super::super::{check_errors, check_ok};

const PRELUDE: &str = "program T;
  type UserId = distinct integer;
  type OrderId = distinct integer;
  type Price = distinct real;
  const Admin: UserId := UserId(1);
  const Low: UserId := UserId(10);
  const High: UserId := UserId(20);
  function Load(): integer; begin return 1; end function;";

fn program(body: &str) -> String {
    format!("{PRELUDE}\nbegin\n{body}\nend.")
}

#[test]
fn distinct_selectors_accept_typed_constants_ranges_bindings_and_nested_values() {
    check_ok(&program(
        "const Id: UserId := UserId(5);
         case Id of
           when Admin: null;
           when UserId(2), UserId(3): null;
           when UserId(4)..UserId(9): null;
           when Low..High: null;
           when const Other if Other > UserId(100): null;
           else null;
         end case;
         case Some(Id) of
           when Some(UserId(1)): null;
           when Some(Low): null;
           when Some(_): null;
           when None: null;
         end case;
         if Some(Id) is Some(UserId(5)) then null; end if;",
    ));
}

#[test]
fn labels_must_have_the_selector_distinct_type() {
    for (label, message) in [
        ("1", "expected `UserId`, found `integer`"),
        ("OrderId(2)", "expected `UserId`, found `OrderId`"),
        ("UserId(1.5)", "expected `integer`, found `real`"),
        ("UserId(1, 2)", "takes exactly one constant value"),
    ] {
        let errors = check_errors(&program(&format!(
            "case UserId(5) of when {label}: null; else null; end case;"
        )));
        assert_eq!(errors.len(), 1, "{label}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{label}: {errors:#?}");
        assert!(errors[0].message.contains(message), "{label}: {errors:#?}");
    }
    let errors = check_errors(&program(
        "case 5 of when UserId(5): null; else null; end case;",
    ));
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0]
            .message
            .contains("expected `integer`, found `UserId`"),
        "{errors:#?}"
    );
}

#[test]
fn label_values_must_be_compile_time_constants() {
    let errors = check_errors(&program(
        "const Computed: UserId := UserId(Load());
         case UserId(5) of when Computed: null; else null; end case;",
    ));
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_NON_CONSTANT_EXPRESSION, "{errors:#?}");
}

#[test]
fn distinct_real_is_not_a_case_selector() {
    let errors = check_errors(&program(
        "case Price(1.0) of when Price(1.0): null; else null; end case;",
    ));
    assert!(
        errors.iter().any(|error| error
            .message
            .contains("Distinct type `Price` cannot be a case selector")),
        "{errors:#?}"
    );
}
