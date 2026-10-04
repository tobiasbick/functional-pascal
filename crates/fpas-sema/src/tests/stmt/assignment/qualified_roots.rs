//! Assignment permissions must follow the resolved unit storage root.

use fpas_diagnostics::codes::{SEMA_IMMUTABLE_ASSIGNMENT, SEMA_UNKNOWN_NAME};
use fpas_parser::{CompilationUnit, parse, parse_compilation_unit};
use fpas_unit::interface::{UnitInterface, decode_interface, encode_interface};

const STATE: &str = r#"unit App.State;
public type Holder of (T) = record public Value: T; end record;
public type Model = Holder of (array of (integer));
public type HiddenModel = record public Value: integer; Secret: integer; end record;
public const Count: integer := 1;
public const Items: array of (integer) := [1];
public const Data: Model := Model(Value := [1]);
public const Mapping: dict of (string, array of (Model)) := ['key': [Model(Value := [1])]];
public const Hidden: HiddenModel := HiddenModel(Value := 1, Secret := 2);
public  var Writable: integer := 1;
public  var MutableItems: array of (integer) := [1];
public  var MutableData: Model := Model(Value := [1]);
public  var MutableMapping: dict of (string, array of (Model)) := ['key': [Model(Value := [1])]];
end unit;"#;

fn state_interface() -> UnitInterface {
    let (parsed, errors) = parse_compilation_unit(STATE);
    assert!(errors.is_empty(), "{errors:#?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("fixture must be a unit");
    };
    let analysis = crate::analyze_unit(&unit, &[]).expect("unit analysis");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:#?}",
        analysis.metadata.errors
    );
    let encoded = encode_interface(&analysis.interface.expect("unit interface")).expect("encode");
    decode_interface(&encoded).expect("decode")
}

fn errors_for(body: &str) -> Vec<crate::SemaError> {
    let (program, errors) = parse(&format!(
        "program Main; uses App.State as Store; begin {body} end program;"
    ));
    assert!(errors.is_empty(), "{errors:#?}");
    crate::analyze_program_with_interfaces(&program, &[state_interface()])
        .expect("program analysis")
        .errors
}

#[test]
fn immutable_imported_paths_report_one_compile_time_assignment_error() {
    for target in [
        "Store.Count",
        "sToRe.Items[0]",
        "Store.Items[-1]",
        "Store.Items[9223372036854775807]",
        "Store.Data.Value[0]",
        "Store.Mapping['key'][0].Value[0]",
    ] {
        let errors = errors_for(&format!("{target} := 2;"));
        assert_eq!(errors.len(), 1, "{target}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_IMMUTABLE_ASSIGNMENT, "{target}");
        assert!(
            errors[0].message.starts_with("Cannot assign to"),
            "{errors:#?}"
        );
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|help| help.contains("var")),
            "{errors:#?}"
        );
    }
}

#[test]
fn mutable_imported_paths_and_independent_copies_are_writable() {
    let errors = errors_for(
        "Store.Writable := 2;
         Store.MutableItems[0] := 2;
         Store.MutableData.Value[0] := 2;
         Store.MutableMapping['key'][0].Value[0] := 2;
         var Copy: array of (integer) := Store.Items;
         Copy[0] := 2;",
    );
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn invalid_imported_paths_preserve_name_type_and_visibility_diagnostics() {
    for target in [
        "Store.Missing",
        "Store.Items[MissingIndex()]",
        "Store.Hidden.Secret",
    ] {
        let errors = errors_for(&format!("{target} := 2;"));
        assert_eq!(errors.len(), 1, "{target}: {errors:#?}");
        assert_ne!(errors[0].code, SEMA_IMMUTABLE_ASSIGNMENT, "{target}");
    }
    let errors = errors_for("Store.Items[MissingIndex()] := MissingValue();");
    assert_eq!(errors.len(), 2, "{errors:#?}");
    assert!(
        errors.iter().all(|error| error.code == SEMA_UNKNOWN_NAME),
        "{errors:#?}"
    );
}

#[test]
fn nested_routines_resolve_the_same_imported_root() {
    let (program, errors) = parse(
        "program Main; uses App.State as Store;
         procedure Change(); begin Store.Count := 2; end procedure;
         begin Change(); end program;",
    );
    assert!(errors.is_empty(), "{errors:#?}");
    let errors = crate::analyze_program_with_interfaces(&program, &[state_interface()])
        .expect("program analysis")
        .errors;
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_IMMUTABLE_ASSIGNMENT);
}

#[test]
fn selected_storage_requires_the_declared_alias_for_reads_writes_and_var_arguments() {
    for target in [
        "App.State.MutableItems[0]",
        "App.State.MutableData.Value[0]",
        "App.State.MutableMapping['key'][0].Value[0]",
        "aPp.StAtE.MutableData.Value[0]",
    ] {
        for body in [
            format!("var Value: integer := {target};"),
            format!("{target} := 2;"),
            format!(
                r#"const Action: procedure(var Value: integer) := procedure(var Value: integer) begin null; end procedure; Action(var {target});"#
            ),
        ] {
            let errors = errors_for(&body);
            assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
            assert_eq!(errors[0].code, SEMA_UNKNOWN_NAME, "{body}: {errors:#?}");
            assert!(
                errors[0]
                    .help
                    .as_ref()
                    .is_some_and(|help| help.to_ascii_lowercase().contains("store")),
                "{errors:#?}"
            );
        }
    }
}
