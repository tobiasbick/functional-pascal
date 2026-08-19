use fpas_parser::{CompilationUnit, parse_compilation_unit};
use fpas_unit::interface::{InterfaceType, SymbolKind};

use super::analyze_unit;

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
        "unit Demo.Types;
         public const Answer: integer := 42;
         const Secret: integer := 7;
         public type Point = record public X: integer; public Y: integer; end;
         public function GetX(P: Point): integer;
         begin return P.X end;",
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
        "unit Demo.Api;
         public type State = enum Idle; Ready; end;
         public function Next(Value: integer): integer;
         begin return Value + 1 end;",
    );
    let dependency_analysis =
        analyze_unit(&dependency, &[]).expect("dependency analysis must succeed");
    assert!(
        dependency_analysis.metadata.errors.is_empty(),
        "{:#?}",
        dependency_analysis.metadata.errors
    );

    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Api;
         public function Run(Value: integer): integer;
         begin
           var Current: State := State.Ready;
           return Next(Value)
         end;",
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
fn private_body_changes_do_not_change_interface_digest() {
    let left = parse_unit(
        "unit Demo.Stable;
         public function PublicValue(X: integer): integer;
         begin return X end;
         function Hidden(): integer;
         begin return 1 end;",
    );
    let right = parse_unit(
        "unit Demo.Stable;
         public function PublicValue(X: integer): integer;
         begin return X + 99 end;
         function Hidden(): integer;
         begin return 2 end;",
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
        "unit Demo.First;
         public function Value(): integer;
         begin return 1 end;",
    );
    let second = parse_unit(
        "unit Demo.Second;
         public function Value(): integer;
         begin return 2 end;",
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
        "unit Demo.Qualified;
         uses Demo.First, Demo.Second;
         public function Run(): integer;
         begin return Demo.First.Value() + Demo.Second.Value() end;",
    );
    let qualified_analysis = analyze_unit(&qualified, &interfaces).expect("qualified analysis");
    assert!(
        qualified_analysis.metadata.errors.is_empty(),
        "{:#?}",
        qualified_analysis.metadata.errors
    );

    let ambiguous = parse_unit(
        "unit Demo.Ambiguous;
         uses Demo.First, Demo.Second;
         public function Run(): integer;
         begin return Value() end;",
    );
    let ambiguous_analysis = analyze_unit(&ambiguous, &interfaces).expect("ambiguous analysis");
    assert_eq!(ambiguous_analysis.metadata.errors.len(), 1);
    assert!(
        ambiguous_analysis.metadata.errors[0]
            .message
            .contains("Ambiguous imported symbol `Value`")
    );
    assert!(ambiguous_analysis.interface.is_none());
}

#[test]
fn imported_enum_type_qualified_short_variant_is_ambiguous() {
    let first = parse_unit(
        "unit Demo.First;
         public type Color = enum Red; Blue; end;",
    );
    let second = parse_unit(
        "unit Demo.Second;
         public type Color = enum Red; Green; end;",
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
        "unit Demo.Qualified;
         uses Demo.First, Demo.Second;
         public function Run(): Demo.First.Color;
         begin return Demo.First.Color.Red end;",
    );
    let qualified_analysis = analyze_unit(&qualified, &interfaces).expect("qualified analysis");
    assert!(
        qualified_analysis.metadata.errors.is_empty(),
        "{:#?}",
        qualified_analysis.metadata.errors
    );

    let ambiguous = parse_unit(
        "unit Demo.Ambiguous;
         uses Demo.First, Demo.Second;
         public function Run(): Demo.First.Color;
         begin return Color.Red end;",
    );
    let ambiguous_analysis = analyze_unit(&ambiguous, &interfaces).expect("ambiguous analysis");
    assert!(
        ambiguous_analysis.metadata.errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_AMBIGUOUS_IMPORTED_NAME
                && error.help.as_deref().is_some_and(|help| {
                    help.contains("Demo.First.Color") && help.contains("Demo.Second.Color")
                })
        }),
        "{:#?}",
        ambiguous_analysis.metadata.errors
    );
}
