//! Optional handlers obey ordinary record construction and binding rules.
//!
//! **Documentation:** `docs/pascal/language/functions/first-class.md`

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT;

const HANDLERS: &str = "
type Button = record
  Id: integer;
  OnClick: Option of procedure(Sender: Button) := None;
  procedure RaiseClick(Self: Button);
  begin
    if Self.OnClick is Some(const Handler) then Handler(Self); end if;
  end procedure;
end record;
procedure Handle(Sender: Button); begin null; end procedure;
";

#[test]
fn optional_handlers_accept_assignment_query_and_calls() {
    check_ok(&format!(
        "program T; {HANDLERS} begin
           var B: Button := Button(Id := 1);
           B.OnClick := Some(Handle);
           if B.OnClick is Some(_) then B.RaiseClick(); end if;
           B.OnClick := (None);
         end."
    ));
}

#[test]
fn optional_handlers_are_constructor_and_update_fields() {
    check_ok(&format!(
        "program T; {HANDLERS} begin
           const B: Button := Button(Id := 1, OnClick := Some(Handle));
           const C: Button := B with OnClick := None; end with;
           B.RaiseClick();
           C.RaiseClick();
         end."
    ));
}

#[test]
fn optional_handler_assignment_requires_a_mutable_record() {
    let errors = check_errors(&format!(
        "program T; {HANDLERS} begin
           const B: Button := Button(Id := 1);
           B.OnClick := Some(Handle);
         end."
    ));
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_IMMUTABLE_ASSIGNMENT);
    assert!(
        errors[0]
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("`const`")),
        "{errors:#?}"
    );
}
