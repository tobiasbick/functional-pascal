//! Record event semantic tests.
//!
//! **Documentation:** `docs/pascal/language/types/record-events.md`

use super::super::{check_errors, check_ok};

fn event_prelude() -> &'static str {
    "program T;\ntype\n  Button = record\n    Id: integer;\n    function ReadOnClick(Self: Button): Option of (procedure(Sender: Button));\n    begin\n      return Option.None;\n    end function;\n    procedure WriteOnClick(Self: Button; Handler: Option of (procedure(Sender: Button)));\n    begin\n      null;\n    end procedure;\n    event OnClick: procedure(Sender: Button) read ReadOnClick write WriteOnClick;\n    procedure RaiseClick(Self: Button);\n    begin\n      if Assigned(Self.OnClick) then\n        Self.OnClick(Self);\n      end if;\n    end procedure;\n  end record;\n"
}

#[test]
fn event_assign_assigned_and_raise_ok() {
    check_ok(&format!(
        "{}procedure Handle(Sender: Button);\nbegin\n  null;\nend procedure;\nbegin\n  var B: Button := Button(Id := 1);\n  B.OnClick := Handle;\n  if Assigned(B.OnClick) then\n    B.RaiseClick();\n  end if;\n  B.OnClick := nil;\nend program;",
        event_prelude()
    ));
}

#[test]
fn bare_event_read_is_rejected() {
    let errors = check_errors(&format!(
        "{}begin\n  var B: Button := Button(Id := 1);\n  var H: procedure(Sender: Button) := B.OnClick;\nend program;",
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
    let errors = check_errors(r#"program T;  var X: integer := 0; begin X := nil; end program;"#);
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
        r#"program T;

  type Button = record
    function ReadOnClick(Self: Button): procedure();
    begin return procedure() begin null; end procedure;
    end function;
    procedure WriteOnClick(Self: Button; Handler: procedure());
    begin null;
    end procedure;
    event OnClick: procedure() read ReadOnClick write WriteOnClick;
  end record;
begin null; end program;"#,
    );
    assert!(
        errors.iter().any(|e| e.message.contains("Option of")),
        "{errors:#?}"
    );
}

#[test]
fn event_duplicate_member_name_rejected() {
    let errors = check_errors(
        r#"program T;

  type Button = record
    OnClick: integer;
    function ReadOnClick(Self: Button): Option of (procedure());
    begin return Option.None;
    end function;
    procedure WriteOnClick(Self: Button; Handler: Option of (procedure()));
    begin null;
    end procedure;
    event OnClick: procedure() read ReadOnClick write WriteOnClick;
  end record;
begin null; end program;"#,
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
        r#"program T;

  type Button = record
    function ReadOnClick of (T)(Self: Button): Option of (procedure());
    begin return Option.None;
    end function;
    procedure WriteOnClick(Self: Button; mutable Handler: Option of (procedure()));
    begin null;
    end procedure;
    event OnClick: procedure() read ReadOnClick write WriteOnClick;
  end record;
begin null; end program;"#,
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
        "{}procedure Handle(Sender: Button);\nbegin\n  null;\nend procedure;\nbegin\n  var B: Button := Button(Id := 1, OnClick := Handle);\n  var C: Button := B with OnClick := Handle; end with;\nend program;",
        event_prelude()
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.message == "Record type `Button` has no field `OnClick`"),
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
        "{}begin\n  var B: Button := Button(Id := 1);\n  go B.OnClick(B);\nend program;",
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
        "{}begin\n  var B: Button := Button(Id := 1);\n  B.OnClick := (nil);\nend program;",
        event_prelude()
    ));
}
