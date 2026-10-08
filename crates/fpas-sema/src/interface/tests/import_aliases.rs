//! Source-local alias visibility, canonical interfaces, and namespace diagnostics.

use super::*;
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_unit::interface::UnitInterface;

fn api() -> UnitInterface {
    let source = "unit Demo.Api;
        public const Answer: integer := 42;
        public var Total: integer := 0;
        public type State = enum Idle; Ready; end enum;
        public type Shape = enum Rect(Width: integer; Height: integer); end enum;
        public type Step = procedure(var Value: integer);
        public type Point = record public X: integer; end record;
        public function Next(Value: integer): integer;
        begin return Value + 1; end function;
        public procedure Increase(var Value: integer);
        begin Value := Value + 1; end procedure;
        end unit;";
    let result = analyze_unit(&parse_unit(source), &[]).expect("API analysis");
    assert!(
        result.metadata.errors.is_empty(),
        "{:#?}",
        result.metadata.errors
    );
    result.interface.expect("API interface")
}

fn errors(imports: &str, declarations: &str, body: &str) -> Vec<crate::SemaError> {
    let source = format!(
        "unit Demo.Consumer; uses {imports}; {declarations} public procedure Run(); begin {body} end procedure; end unit;"
    );
    analyze_unit(&parse_unit(&source), &[api()])
        .expect("consumer analysis")
        .metadata
        .errors
}

#[test]
fn alias_resolves_exports_without_changing_nominal_type_identity() {
    let source = "unit Demo.Consumer; uses Demo.Api as Api;
        public type Point = Api.Point;
        public function Run(): integer;
        begin
          var P: aPI.Point := record X := Api.Answer; end;
          const S: Api.State := Api.State.Ready;
          const Shape: Api.Shape := Api.Shape.Rect(Height := 3, Width := 4);
          const Step: Api.Step := Api.Increase;
          Step(var P.X);
          Api.Increase(var Api.Total);
          return Api.Next(Value := P.X);
        end function;
        end unit;";
    let result = analyze_unit(&parse_unit(source), &[api()]).expect("consumer analysis");
    assert!(
        result.metadata.errors.is_empty(),
        "{:#?}",
        result.metadata.errors
    );
    assert_eq!(
        result
            .metadata
            .import_aliases
            .get("demo.api")
            .map(String::as_str),
        Some("Api")
    );
    let exported = result.interface.expect("consumer interface");
    let point = exported
        .symbols
        .iter()
        .find(|symbol| symbol.name == "Point")
        .expect("Point");
    let InterfaceType::Record(record) = &point.ty else {
        panic!("record alias");
    };
    assert_eq!(record.name, "demo.api.point");
}

#[test]
fn alias_hides_original_paths_and_all_short_names() {
    for body in [
        "const X: integer := Next(1);",
        "const X: integer := Answer;",
        "const X: State := Ready;",
        "const X: Shape := Shape.Rect(1, 2);",
        "const X: Demo.Api.Point := record X := 1; end;",
        "const X: integer := Demo.Api.Next(1);",
        "const X: integer := Demo.Api.Answer;",
    ] {
        let diagnostics = errors("Demo.Api as Api", "", body);
        assert!(!diagnostics.is_empty(), "accepted {body}");
        if body.contains("Demo.Api") {
            assert!(
                diagnostics.iter().any(|error| error
                    .help
                    .as_deref()
                    .unwrap_or_default()
                    .contains("imported as `Api`")),
                "{body}: {diagnostics:#?}"
            );
        }
    }
}

#[test]
fn repeated_units_are_rejected_in_every_plain_alias_combination() {
    for imports in [
        "Demo.Api, demo.api",
        "Demo.Api as Api, DEMO.API as Other",
        "Demo.Api, Demo.Api as Api",
        "Demo.Api as Api, Demo.Api",
    ] {
        let diagnostics = errors(imports, "", "");
        let repeated = diagnostics
            .iter()
            .find(|error| error.message.contains("Repeated import"))
            .expect("repeated-import error");
        assert_eq!(repeated.code, SEMA_DUPLICATE_DECLARATION);
        assert!(
            repeated.span.expect("source span").offset()
                > "unit Demo.Consumer; uses Demo.Api".len()
        );
        assert!(
            repeated
                .help
                .as_deref()
                .unwrap_or_default()
                .contains("once per source file")
        );
    }
}

#[test]
fn aliases_cannot_conflict_with_unit_roots_primitives_or_each_other() {
    for imports in [
        "Demo.Api as Std",
        "Demo.Api as Demo",
        "Demo.Api as integer",
        "Demo.Api as Api, Std.Math as aPI",
    ] {
        let diagnostics = errors(imports, "", "");
        assert!(
            diagnostics
                .iter()
                .any(|error| error.code == SEMA_DUPLICATE_DECLARATION
                    && error.message.contains("Import alias")),
            "{imports}: {diagnostics:#?}"
        );
    }
}

#[test]
fn alias_cannot_be_shadowed_by_declarations_parameters_loops_or_patterns() {
    for (declarations, body) in [
        ("const Api: integer := 1;", ""),
        ("type Api = integer;", ""),
        ("procedure Api(); begin end procedure;", ""),
        ("procedure F(Api: integer); begin end procedure;", ""),
        (
            "function F<Api>(Value: Api): Api; begin return Value; end function;",
            "",
        ),
        ("", "var Api: integer := 1;"),
        ("", "const Api: integer := 1;"),
        ("", "for Api: integer := 1 to 2 do null; end for;"),
        ("", "for Api: integer in [1] do null; end for;"),
        (
            "",
            "const F: function(Value: integer): integer := function(Api: integer): integer begin return Api; end function;",
        ),
        (
            "",
            "const X: option of integer := Some(1); case X of when Some(Api): null; when None: null; end case;",
        ),
    ] {
        let diagnostics = errors("Demo.Api as Api", declarations, body);
        assert!(
            diagnostics
                .iter()
                .any(|error| error.code == SEMA_DUPLICATE_DECLARATION
                    && error.message.contains("import alias")
                    && error.help.as_deref().unwrap_or_default().contains("Rename")),
            "{declarations} {body}: {diagnostics:#?}"
        );
    }
}

#[test]
fn unrelated_plain_import_does_not_change_alias_lookup_or_open_its_short_names() {
    let math = analyze_unit(&parse_unit("unit Other.Math; public function Next(Value: integer): integer; begin return Value * 2; end function; end unit;"), &[]).expect("analysis").interface.expect("interface");
    let consumer = parse_unit(
        "unit Demo.Consumer; uses Demo.Api as Api, Other.Math; public function Run(): integer; begin return Api.Next(1) + Next(1); end function; end unit;",
    );
    let result = analyze_unit(&consumer, &[api(), math]).expect("analysis");
    assert!(
        result.metadata.errors.is_empty(),
        "{:#?}",
        result.metadata.errors
    );
}

#[test]
fn intrinsic_aliases_require_alias_qualification_and_preserve_contextual_as() {
    let consumer = parse_unit(
        "unit Demo.Consumer; uses Std.Math as As; public function Run(): real; begin return aS.Sqrt(9.0) + as.Pi; end function; end unit;",
    );
    let result = analyze_unit(&consumer, &[]).expect("analysis");
    assert!(
        result.metadata.errors.is_empty(),
        "{:#?}",
        result.metadata.errors
    );
    for body in [
        "const X: real := Sqrt(9.0);",
        "const X: real := Std.Math.Sqrt(9.0);",
        "const X: real := Pi;",
    ] {
        assert!(
            !errors("Std.Math as Math", "", body).is_empty(),
            "accepted {body}"
        );
    }
}

#[test]
fn alias_does_not_expose_nested_units_and_keeps_their_plain_paths_visible() {
    let child = analyze_unit(&parse_unit("unit Demo.Api.Extra; public function Value(): integer; begin return 5; end function; end unit;"), &[]).expect("child analysis").interface.expect("child interface");
    for (expression, valid) in [
        ("Api.Next(1) + Demo.Api.Extra.Value()", true),
        ("Api.Extra.Value()", false),
    ] {
        let source = format!(
            "unit Demo.Consumer; uses Demo.Api as Api, Demo.Api.Extra; public function Run(): integer; begin return {expression}; end function; end unit;"
        );
        let result =
            analyze_unit(&parse_unit(&source), &[api(), child.clone()]).expect("consumer analysis");
        assert_eq!(
            result.metadata.errors.is_empty(),
            valid,
            "{expression}: {:#?}",
            result.metadata.errors
        );
    }
}

#[test]
fn alias_does_not_expose_types_from_transitive_supporting_units() {
    let child = analyze_unit(&parse_unit("unit Demo.Api.Internal; public type Secret = record public Value: integer; end record; end unit;"), &[]).expect("support analysis").interface.expect("support interface");
    let consumer = parse_unit(
        "unit Demo.Consumer; uses Demo.Api as Api; public type Leak = Api.Internal.Secret; end unit;",
    );
    let result = super::super::analyze_unit_with_interface_support(&consumer, &[api()], &[child])
        .expect("consumer analysis");
    assert!(
        !result.metadata.errors.is_empty(),
        "supporting types are not unit exports"
    );
}
