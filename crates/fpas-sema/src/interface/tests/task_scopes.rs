//! Imported parameter escape contracts survive persistent unit interfaces.

use super::{analyze_unit, parse_unit};
use fpas_diagnostics::codes::SEMA_TASK_HANDLE_ESCAPE;
use fpas_unit::interface::{decode_interface, encode_interface};

#[test]
fn imported_helpers_enforce_escaping_parameters_after_interface_encoding() {
    let dependency = parse_unit(
        "unit Demo.Scopes;
uses Std.Tasks;
var Kept: array of task of integer := [];
public function Read(H: task of integer): integer;
  function Take(): integer; begin return Wait(H); end function;
begin return Take(); end function;
public procedure Save(H: task of integer);
  procedure Store(); begin Kept.Push(H); end procedure;
begin Store(); end procedure;
public type Reader = record
  public Value: integer;
  public function Read(Self: Reader; H: task of integer): integer;
  begin return Wait(H) + Self.Value; end function;
end record;
end unit;",
    );
    let analysis = analyze_unit(&dependency, &[]).expect("dependency");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.expect("interface");
    let interface =
        decode_interface(&encode_interface(&interface).expect("encode")).expect("decode");
    for (body, rejected) in [
        ("discard API.Read(Job);", false),
        ("API.Save(Job);", true),
        (
            "const R: API.Reader := API.Reader(Value := 1); discard R.Read(Job);",
            false,
        ),
        (
            "const ReadValue: function(H: task of integer): integer := API.Read; discard ReadValue(Job);",
            true,
        ),
    ] {
        let consumer = parse_unit(&format!("unit Demo.Consumer;
uses Demo.Scopes as API;
function Fetch(): integer; begin return 1; end function;
public procedure Run(); begin scope const Job: task of integer := go Fetch(); {body} end scope; end procedure;
end unit;"));
        let analysis = analyze_unit(&consumer, std::slice::from_ref(&interface)).expect("consumer");
        let errors = &analysis.metadata.errors;
        if rejected {
            assert!(
                errors
                    .iter()
                    .any(|error| error.code == SEMA_TASK_HANDLE_ESCAPE),
                "{body}: {errors:?}"
            );
            assert!(
                errors
                    .iter()
                    .all(|error| error.code == SEMA_TASK_HANDLE_ESCAPE),
                "{body}: {errors:?}"
            );
        } else {
            assert!(errors.is_empty(), "{body}: {errors:?}");
        }
    }
}

#[test]
fn changed_parameter_escapes_change_the_unit_interface_digest() {
    let interface = |body: &str| {
        let unit = parse_unit(&format!(
            "unit Demo.Contract; uses Std.Tasks;
var Kept: array of task of integer := [];
public procedure Use(H: task of integer); begin {body} end procedure;
end unit;"
        ));
        let analysis = analyze_unit(&unit, &[]).expect("analysis");
        assert!(
            analysis.metadata.errors.is_empty(),
            "{:?}",
            analysis.metadata.errors
        );
        analysis.interface.expect("interface")
    };
    let safe = interface("discard Wait(H);");
    let escaping = interface("Kept.Push(H);");
    assert_ne!(
        safe.digest().expect("digest"),
        escaping.digest().expect("digest")
    );
}
