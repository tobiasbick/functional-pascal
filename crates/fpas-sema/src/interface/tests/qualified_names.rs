//! Mandatory qualifiers across imported source interfaces and intrinsic units.

use super::*;
use fpas_diagnostics::codes::SEMA_UNQUALIFIED_NAME;

fn interface_of(source: &str) -> fpas_unit::interface::UnitInterface {
    analyze_unit(&parse_unit(source), &[])
        .expect("dependency analysis")
        .interface
        .expect("dependency interface")
}

#[test]
fn native_registration_never_opens_short_names_from_imported_interfaces() {
    let sender = interface_of(
        "unit Demo.Sender; public function Send(A: integer; B: integer; C: integer): integer; begin return A + B + C; end function; end unit;",
    );
    for imports in [
        "Demo.Sender as Sender",
        "Demo.Sender as Sender, Std.Tasks as Jobs, Std.Math as Math",
    ] {
        let prefix = if imports.contains("Std.Math") {
            "discard Math.Abs(-1);"
        } else {
            ""
        };
        let source = format!(
            "unit Demo.Consumer; uses {imports}; public function Run(): integer; begin {prefix} return Send(1, 2, 3); end function; end unit;"
        );
        let errors = analyze_unit(&parse_unit(&source), std::slice::from_ref(&sender))
            .unwrap()
            .metadata
            .errors;
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_UNQUALIFIED_NAME);
        assert!(
            errors[0]
                .help
                .as_ref()
                .is_some_and(|help| help.contains("Sender.Send")),
            "{errors:#?}"
        );
        let repaired = source.replace("return Send(", "return Sender.Send(");
        assert!(
            analyze_unit(&parse_unit(&repaired), std::slice::from_ref(&sender))
                .unwrap()
                .metadata
                .errors
                .is_empty()
        );
    }
}

#[test]
fn imported_types_routines_and_variants_with_the_same_leaf_keep_distinct_targets() {
    let frames = interface_of(
        "unit Demo.Frames; public type Frame = record public X: integer; end record; end unit;",
    );
    let routines = interface_of(
        "unit Demo.Routines; public function Frame(X: integer): integer; begin return X; end function; end unit;",
    );
    let signals = interface_of(
        "unit Demo.Signals; public type Signal = enum Frame(Milliseconds: integer); Other; end enum; end unit;",
    );
    let interfaces = [frames, routines, signals];
    let source = "unit Demo.Consumer; uses Demo.Frames as Frames, Demo.Routines as Routines, Demo.Signals as Signals;
        public function Run(): integer;
        begin const F: Frames.Frame := Frames.Frame(X := 1);
          const S: Signals.Signal := Signals.Signal.Frame(Milliseconds := 2);
          discard S is Signals.Signal.Frame(_);
          return Routines.Frame(F.X);
        end function; end unit;";
    let result = analyze_unit(&parse_unit(source), &interfaces).unwrap();
    assert!(
        result.metadata.errors.is_empty(),
        "{:#?}",
        result.metadata.errors
    );
    for (before, after, repair) in [
        ("Routines.Frame(F.X)", "Frame(F.X)", "Routines.Frame"),
        (
            "Signals.Signal.Frame(Milliseconds := 2)",
            "Signals.Frame(Milliseconds := 2)",
            "Signals.signal.Frame",
        ),
        ("Frames.Frame :=", "Frame :=", "Frames.Frame"),
    ] {
        let result =
            analyze_unit(&parse_unit(&source.replace(before, after)), &interfaces).unwrap();
        assert!(
            result
                .metadata
                .errors
                .iter()
                .any(|error| error.code == SEMA_UNQUALIFIED_NAME
                    && error.help.as_ref().is_some_and(|help| help
                        .to_ascii_lowercase()
                        .contains(&repair.to_ascii_lowercase()))),
            "{:#?}",
            result.metadata.errors
        );
    }
}

#[test]
fn ancestor_units_and_exported_members_keep_their_exact_owning_alias() {
    let parent = interface_of("unit A; public type B = enum C; end enum; end unit;");
    let child = interface_of("unit A.B; public const C: integer := 7; end unit;");
    let consumer = parse_unit(
        "unit Consumer; uses A as Root, A.B as Child;
        public function Run(): integer;
        begin const Value: Root.B := Root.B.C;
          const Number: integer := Child.C;
          return case Value of when Root.B.C: Number; end case;
        end function; end unit;",
    );
    for interfaces in [vec![parent.clone(), child.clone()], vec![child, parent]] {
        let analysis = analyze_unit(&consumer, &interfaces).unwrap();
        assert!(
            analysis.metadata.errors.is_empty(),
            "{:#?}",
            analysis.metadata.errors
        );
    }
}
