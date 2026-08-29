use fpas_diagnostics::codes::SEMA_PRIVATE_TYPE_IN_PUBLIC_SIGNATURE;

use super::{analyze_unit, parse_unit};

fn assert_private_signature_error(source: &str, declaration: &str, private_type: &str) {
    let analysis = analyze_unit(&parse_unit(source), &[]).expect("unit analysis must succeed");
    let matching = analysis
        .metadata
        .errors
        .iter()
        .filter(|error| error.code == SEMA_PRIVATE_TYPE_IN_PUBLIC_SIGNATURE)
        .collect::<Vec<_>>();

    assert_eq!(
        matching.len(),
        1,
        "unexpected diagnostics: {:#?}",
        analysis.metadata.errors
    );
    assert!(
        matching[0].message.contains(declaration) && matching[0].message.contains(private_type),
        "diagnostic must name the declaration and private type: {:#?}",
        matching[0]
    );
    assert!(
        matching[0]
            .help
            .as_deref()
            .is_some_and(|help| { help.contains("public") && help.contains("stop exporting") }),
        "diagnostic must explain both repairs: {:#?}",
        matching[0]
    );
    assert!(
        analysis.interface.is_none(),
        "an invalid public interface must not be emitted"
    );
}

#[test]
fn private_record_function_result_is_rejected() {
    assert_private_signature_error(
        "unit Demo.ReturnValue;
         type Hidden = record Value: integer; end;
         public function Make(): Hidden;
         begin return record Value := 1; end end;",
        "Make",
        "Hidden",
    );
}

#[test]
fn private_enum_procedure_parameter_is_rejected() {
    assert_private_signature_error(
        "unit Demo.Parameter;
         type HiddenState = enum Ready; Done; end;
         public procedure Accept(Value: HiddenState);
         begin end;",
        "Accept",
        "HiddenState",
    );
}

#[test]
fn private_record_public_global_is_rejected() {
    assert_private_signature_error(
        "unit Demo.Global;
         type Hidden = record Value: integer; end;
         public var Current: Hidden := record Value := 1; end;",
        "Current",
        "Hidden",
    );
}

#[test]
fn public_alias_of_private_type_is_rejected() {
    assert_private_signature_error(
        "unit Demo.Alias;
         type Hidden = record Value: integer; end;
         public type Exposed = Hidden;",
        "Exposed",
        "Hidden",
    );
}

#[test]
fn private_type_nested_in_callable_collection_is_rejected_once() {
    assert_private_signature_error(
        "unit Demo.Nested;
         type Hidden = record Value: integer; end;
         public procedure Register(
           Callback: function(Values: array of Hidden): Option of Hidden
         );
         begin end;",
        "Register",
        "Hidden",
    );
}

#[test]
fn private_type_in_exported_record_layout_is_rejected() {
    assert_private_signature_error(
        "unit Demo.RecordLayout;
         type Hidden = record Value: integer; end;
         public type Wrapper = record HiddenValue: Hidden; end;",
        "Wrapper",
        "Hidden",
    );
}

#[test]
fn private_type_in_exported_enum_layout_is_rejected() {
    assert_private_signature_error(
        "unit Demo.EnumLayout;
         type Hidden = record Value: integer; end;
         public type Wrapper = enum Value(Item: Hidden); Empty; end;",
        "Wrapper",
        "Hidden",
    );
}

#[test]
fn public_type_in_public_signature_remains_valid() {
    let analysis = analyze_unit(
        &parse_unit(
            "unit Demo.Valid;
             public type Visible = record public Value: integer; end;
             public procedure Accept(Values: array of Visible);
             begin end;",
        ),
        &[],
    )
    .expect("unit analysis must succeed");

    assert!(
        analysis.metadata.errors.is_empty(),
        "{:#?}",
        analysis.metadata.errors
    );
    assert!(
        analysis.interface.is_some(),
        "valid public interface must be emitted"
    );
}

#[test]
fn qualified_import_with_private_local_short_name_remains_valid() {
    let dependency = analyze_unit(
        &parse_unit(
            "unit Demo.Dependency;
             public type Hidden = record public Value: integer; end;",
        ),
        &[],
    )
    .expect("dependency analysis must succeed")
    .interface
    .expect("dependency interface must be valid");
    let analysis = analyze_unit(
        &parse_unit(
            "unit Demo.Consumer;
             uses Demo.Dependency;
             type Hidden = record Value: string; end;
             public procedure Accept(Value: Demo.Dependency.Hidden);
             begin end;",
        ),
        &[dependency],
    )
    .expect("consumer analysis must succeed");

    assert!(
        analysis.metadata.errors.is_empty(),
        "{:#?}",
        analysis.metadata.errors
    );
    assert!(
        analysis.interface.is_some(),
        "qualified public import must remain exportable"
    );
}
