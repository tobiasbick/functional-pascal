use fpas_parser::{CompilationUnit, parse_compilation_unit};
use fpas_unit::interface::{InterfaceType, SymbolKind};

use super::analyze_unit;

mod aliases;
mod public_signatures;
mod short_names;

fn parse_unit(source: &str) -> fpas_parser::Unit {
    let (parsed, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "unexpected parse errors: {errors:#?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("fixture must parse as a unit");
    };
    unit
}

#[test]
fn unit_interface_exports_public_symbols_and_qualified_types() {
    let unit = parse_unit(
        r#"unit Demo.Types;
           public const Answer: integer := 42;
          const Secret: integer := 7;
           public type Point = record public X: integer; public Y: integer; end record;
         public function GetX(P: Point): integer;
         begin return P.X; end function;
end unit;
"#,
    );

    let analysis = analyze_unit(&unit, &[]).expect("unit analysis must succeed");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:#?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.expect("valid interface");
    assert_eq!(
        interface
            .symbols
            .iter()
            .map(|symbol| symbol.name.as_str())
            .collect::<Vec<_>>(),
        ["Answer", "GetX", "Point"]
    );
    assert!(
        interface
            .symbols
            .iter()
            .all(|symbol| !symbol.name.eq_ignore_ascii_case("Secret"))
    );
    let point = interface
        .symbols
        .iter()
        .find(|symbol| symbol.name == "Point")
        .expect("Point export");
    let InterfaceType::Record(record) = &point.ty else {
        panic!("Point must remain a record");
    };
    assert_eq!(record.name, "demo.types.point");
    assert_eq!(
        interface.symbols[0].kind,
        SymbolKind::Constant(Some(fpas_unit::interface::ConstantValue::Integer(42)))
    );
}

#[test]
fn consumer_analysis_uses_interface_without_dependency_ast() {
    let dependency = parse_unit(
        r#"unit Demo.Api;
           public type State = enum Idle; Ready; end enum;
         public function Next(Value: integer): integer;
         begin return Value + 1; end function;
end unit;
"#,
    );
    let dependency_analysis =
        analyze_unit(&dependency, &[]).expect("dependency analysis must succeed");
    assert!(
        dependency_analysis.metadata.errors.is_empty(),
        "{:#?}",
        dependency_analysis.metadata.errors
    );

    let consumer = parse_unit(
        r#"unit Demo.Consumer;
uses Demo.Api as Api;
         public function Run(Value: integer): integer;
         begin
           var Current: Api.State := Api.State.Ready;
           return Api.Next(Value);
         end function;
end unit;
"#,
    );
    let consumer_analysis = analyze_unit(
        &consumer,
        &[dependency_analysis.interface.expect("dependency interface")],
    )
    .expect("consumer analysis must succeed");
    assert!(
        consumer_analysis.metadata.errors.is_empty(),
        "{:#?}",
        consumer_analysis.metadata.errors
    );
}

#[test]
fn enum_backing_values_survive_export_import_and_alias_export() {
    let dependency = parse_unit(
        r#"unit Demo.Values;
           public type State = enum Idle = 7; Ready; Done = 20; end enum;
end unit;
"#,
    );
    let dependency_interface = analyze_unit(&dependency, &[])
        .expect("dependency analysis must succeed")
        .interface
        .expect("dependency interface");
    let state = dependency_interface
        .symbols
        .iter()
        .find(|symbol| symbol.name == "State")
        .expect("State export");
    let InterfaceType::Enum(state) = &state.ty else {
        panic!("State must remain an enum");
    };
    assert_eq!(
        state
            .variants
            .iter()
            .map(|variant| variant.backing_value)
            .collect::<Vec<_>>(),
        [Some(7), Some(8), Some(20)]
    );

    let consumer = parse_unit(
        r#"unit Demo.Aliases;
uses Demo.Values as Values;
           public type StateAlias = Values.State;
end unit;
"#,
    );
    let consumer_interface = analyze_unit(&consumer, &[dependency_interface])
        .expect("consumer analysis must succeed")
        .interface
        .expect("consumer interface");
    let alias = consumer_interface
        .symbols
        .iter()
        .find(|symbol| symbol.name == "StateAlias")
        .expect("StateAlias export");
    let InterfaceType::Enum(alias) = &alias.ty else {
        panic!("StateAlias must retain its enum descriptor");
    };
    assert_eq!(
        alias
            .variants
            .iter()
            .map(|variant| variant.backing_value)
            .collect::<Vec<_>>(),
        [Some(7), Some(8), Some(20)]
    );
}

#[test]
fn private_body_changes_do_not_change_interface_digest() {
    let left = parse_unit(
        r#"unit Demo.Stable;
         public function PublicValue(X: integer): integer;
         begin return X; end function;
         function Hidden(): integer;
         begin return 1; end function;
end unit;
"#,
    );
    let right = parse_unit(
        r#"unit Demo.Stable;
         public function PublicValue(X: integer): integer;
         begin return X + 99; end function;
         function Hidden(): integer;
         begin return 2; end function;
end unit;
"#,
    );
    let left_interface = analyze_unit(&left, &[])
        .expect("left analysis")
        .interface
        .expect("left interface");
    let right_interface = analyze_unit(&right, &[])
        .expect("right analysis")
        .interface
        .expect("right interface");
    assert_eq!(
        left_interface.digest().expect("left digest"),
        right_interface.digest().expect("right digest")
    );
}

#[test]
fn imported_name_ambiguity_is_reported_only_when_short_name_is_used() {
    let first = parse_unit(
        r#"unit Demo.First;
         public function Value(): integer;
         begin return 1; end function;
end unit;
"#,
    );
    let second = parse_unit(
        r#"unit Demo.Second;
         public function Value(): integer;
         begin return 2; end function;
end unit;
"#,
    );
    let interfaces = [
        analyze_unit(&first, &[])
            .expect("first analysis")
            .interface
            .expect("first interface"),
        analyze_unit(&second, &[])
            .expect("second analysis")
            .interface
            .expect("second interface"),
    ];

    let qualified = parse_unit(
        r#"unit Demo.Qualified;
uses Demo.First as First; uses Demo.Second as Second;
         public function Run(): integer;
         begin return First.Value() + Second.Value(); end function;
end unit;
"#,
    );
    let qualified_analysis = analyze_unit(&qualified, &interfaces).expect("qualified analysis");
    assert!(
        qualified_analysis.metadata.errors.is_empty(),
        "{:#?}",
        qualified_analysis.metadata.errors
    );

    let ambiguous = parse_unit(
        r#"unit Demo.Ambiguous;
uses Demo.First as First; uses Demo.Second as Second;
         public function Run(): integer;
         begin return Value(); end function;
end unit;
"#,
    );
    let ambiguous_analysis = analyze_unit(&ambiguous, &interfaces).expect("ambiguous analysis");
    assert_eq!(ambiguous_analysis.metadata.errors.len(), 1);
    assert!(
        ambiguous_analysis.metadata.errors[0]
            .message
            .contains("Unknown function or procedure `Value`")
    );
    assert!(ambiguous_analysis.interface.is_none());
}

#[test]
fn imported_enum_type_qualified_short_variant_is_ambiguous() {
    let first = parse_unit(
        r#"unit Demo.First;
           public type Color = enum Red; Blue; end enum;
end unit;
"#,
    );
    let second = parse_unit(
        r#"unit Demo.Second;
           public type Color = enum Red; Green; end enum;
end unit;
"#,
    );
    let interfaces = [
        analyze_unit(&first, &[])
            .expect("first analysis")
            .interface
            .expect("first interface"),
        analyze_unit(&second, &[])
            .expect("second analysis")
            .interface
            .expect("second interface"),
    ];

    let qualified = parse_unit(
        r#"unit Demo.Qualified;
uses Demo.First as First; uses Demo.Second as Second;
         public function Run(): First.Color;
         begin return First.Color.Red; end function;
end unit;
"#,
    );
    let qualified_analysis = analyze_unit(&qualified, &interfaces).expect("qualified analysis");
    assert!(
        qualified_analysis.metadata.errors.is_empty(),
        "{:#?}",
        qualified_analysis.metadata.errors
    );

    let ambiguous = parse_unit(
        r#"unit Demo.Ambiguous;
uses Demo.First as First; uses Demo.Second as Second;
         public function Run(): First.Color;
         begin return Color.Red; end function;
end unit;
"#,
    );
    let ambiguous_analysis = analyze_unit(&ambiguous, &interfaces).expect("ambiguous analysis");
    assert!(
        ambiguous_analysis.metadata.errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_UNKNOWN_NAME
                && error.help.as_deref().is_some_and(|help| {
                    help.contains("first.Color") && help.contains("second.Color")
                })
        }),
        "{:#?}",
        ambiguous_analysis.metadata.errors
    );
}
