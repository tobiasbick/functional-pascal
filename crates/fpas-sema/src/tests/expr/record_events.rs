//! Record event semantic tests.
//!
//! **Documentation:** `docs/pascal/language/types/record-events.md`

use super::super::{check_errors, check_ok};

fn event_prelude() -> &'static str {
    "\
program T;
type
  Button = record
    Id: integer;
    function ReadOnClick(Self: Button): Option of procedure(Sender: Button);
    begin
      return None
    end;
    procedure WriteOnClick(Self: Button; Handler: Option of procedure(Sender: Button));
    begin
    end;
    event OnClick: procedure(Sender: Button) read ReadOnClick write WriteOnClick;
    procedure RaiseClick(Self: Button);
    begin
      if Assigned(Self.OnClick) then
        Self.OnClick(Self)
    end;
  end;
"
}

#[test]
fn event_assign_assigned_and_raise_ok() {
    check_ok(&format!(
        "{}\
procedure Handle(Sender: Button);
begin
end;
begin
  var B: Button := record Id := 1; end;
  B.OnClick := Handle;
  if Assigned(B.OnClick) then
    B.RaiseClick();
  B.OnClick := nil
end.",
        event_prelude()
    ));
}

#[test]
fn bare_event_read_is_rejected() {
    let errors = check_errors(&format!(
        "{}\
begin
  var B: Button := record Id := 1; end;
  var H: procedure(Sender: Button) := B.OnClick
end.",
        event_prelude()
    ));
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("Cannot read event")),
        "{errors:#?}"
    );
}

#[test]
fn nil_outside_event_assignment_is_rejected() {
    let errors = check_errors("program T; var X: integer := 0; begin X := nil end.");
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("`nil` is only valid")),
        "{errors:#?}"
    );
}

#[test]
fn event_requires_option_accessors() {
    let errors = check_errors(
        "\
program T;
type
  Button = record
    function ReadOnClick(Self: Button): procedure();
    begin
      return procedure() begin end
    end;
    procedure WriteOnClick(Self: Button; Handler: procedure());
    begin
    end;
    event OnClick: procedure() read ReadOnClick write WriteOnClick;
  end;
begin
end.",
    );
    assert!(
        errors.iter().any(|e| e.message.contains("Option of")),
        "{errors:#?}"
    );
}

#[test]
fn event_duplicate_member_name_rejected() {
    let errors = check_errors(
        "\
program T;
type
  Button = record
    OnClick: integer;
    function ReadOnClick(Self: Button): Option of procedure();
    begin
      return None
    end;
    procedure WriteOnClick(Self: Button; Handler: Option of procedure());
    begin
    end;
    event OnClick: procedure() read ReadOnClick write WriteOnClick;
  end;
begin
end.",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("Duplicate record member")),
        "{errors:#?}"
    );
}

#[test]
fn event_rejects_generic_and_mutable_accessors() {
    let errors = check_errors(
        "\
program T;
type
  Button = record
    function ReadOnClick<T>(Self: Button): Option of procedure();
    begin
      return None
    end;
    procedure WriteOnClick(Self: Button; mutable Handler: Option of procedure());
    begin
    end;
    event OnClick: procedure() read ReadOnClick write WriteOnClick;
  end;
begin
end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("cannot be generic")),
        "{errors:#?}"
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("by value")),
        "{errors:#?}"
    );
}

#[test]
fn event_cannot_be_initialized_or_updated_as_a_field() {
    let errors = check_errors(&format!(
        "{}\
procedure Handle(Sender: Button);
begin
end;
begin
  var B: Button := record Id := 1; OnClick := Handle; end;
  var C: Button := B with OnClick := Handle; end
end.",
        event_prelude()
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("cannot be initialized")),
        "{errors:#?}"
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("cannot be set in a `with` update")),
        "{errors:#?}"
    );
}

#[test]
fn event_raise_cannot_cross_task_boundary() {
    let errors = check_errors(&format!(
        "{}\
begin
  var B: Button := record Id := 1; end;
  go B.OnClick(B)
end.",
        event_prelude()
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("event across a task boundary")),
        "{errors:#?}"
    );
}

#[test]
fn parenthesized_nil_clears_event() {
    check_ok(&format!(
        "{}\
begin
  var B: Button := record Id := 1; end;
  B.OnClick := (nil)
end.",
        event_prelude()
    ));
}
