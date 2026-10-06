//! Short names of imported source units resolve together with `Std.*` short names.

use super::*;

fn interface_of(source: &str) -> fpas_unit::interface::UnitInterface {
    analyze_unit(&parse_unit(source), &[])
        .expect("dependency analysis")
        .interface
        .expect("dependency interface")
}

fn error_messages(source: &str, interfaces: &[fpas_unit::interface::UnitInterface]) -> Vec<String> {
    analyze_unit(&parse_unit(source), interfaces)
        .expect("analysis")
        .metadata
        .errors
        .iter()
        .map(|error| error.message.clone())
        .collect()
}

#[test]
fn source_and_std_short_names_are_ambiguous_even_with_different_arity() {
    let sender = interface_of(
        "unit Demo.Sender;
         public function Send(A: integer; B: integer; C: integer): integer;
         begin return A + B + C; end function;\nend unit;",
    );
    let interfaces = [sender];

    let short = error_messages(
        "unit Demo.Short;
         uses Demo.Sender, Std.Tasks;
         public function Run(): integer;
         begin return Send(1, 2, 3); end function;\nend unit;",
        &interfaces,
    );
    assert_eq!(short.len(), 1, "{short:#?}");
    assert!(
        short[0].contains("Ambiguous imported symbol `Send`"),
        "{short:#?}"
    );

    let qualified = error_messages(
        "unit Demo.Qualified;
         uses Demo.Sender, Std.Tasks;
         public function Run(): integer;
         begin return Demo.Sender.Send(1, 2, 3); end function;\nend unit;",
        &interfaces,
    );
    assert!(qualified.is_empty(), "{qualified:#?}");

    let without_std = error_messages(
        "unit Demo.WithoutStd;
         uses Demo.Sender;
         public function Run(): integer;
         begin return Send(1, 2, 3); end function;\nend unit;",
        &interfaces,
    );
    assert!(without_std.is_empty(), "{without_std:#?}");
}

#[test]
fn qualified_std_use_keeps_source_short_name_ambiguity() {
    let first = interface_of(
        "unit Demo.First;
         public function Value(): integer;
         begin return 1; end function;\nend unit;",
    );
    let second = interface_of(
        "unit Demo.Second;
         public function Value(): integer;
         begin return 2; end function;\nend unit;",
    );
    // Registering the qualified `Std.Math` symbols on first use rebuilds the short names.
    let messages = error_messages(
        "unit Demo.Lazy;
         uses Demo.First, Demo.Second, Std.Math;
         public function Run(): integer;
         begin return Std.Math.Abs(-1) + Value(); end function;\nend unit;",
        &[first, second],
    );
    assert_eq!(messages.len(), 1, "{messages:#?}");
    assert!(
        messages[0].contains("Ambiguous imported symbol `Value`"),
        "{messages:#?}"
    );
}

#[test]
fn imported_type_hides_imported_enum_variant_short_name() {
    let frames = interface_of(
        "unit Demo.Frames;
         public type Frame = record public X: integer; end record;\nend unit;",
    );
    let signals = interface_of(
        "unit Demo.Signals;
         public type Signal = enum Frame(Milliseconds: integer); Other; end enum;\nend unit;",
    );
    let interfaces = [frames, signals];

    // As within one unit, the type name wins and the variant stays reachable qualified.
    let messages = error_messages(
        "unit Demo.Mixed;
         uses Demo.Frames, Demo.Signals;
         public function Run(): integer;
         begin
           var F: Frame := record X := 1; end;
           var S: Signal := Signal.Frame(2);
           return F.X;
         end function;\nend unit;",
        &interfaces,
    );
    assert!(messages.is_empty(), "{messages:#?}");

    let variant_short = error_messages(
        "unit Demo.VariantShort;
         uses Demo.Frames, Demo.Signals;
         public function Run(): Signal;
         begin return Frame(2); end function;\nend unit;",
        &interfaces,
    );
    assert_eq!(variant_short.len(), 1, "{variant_short:#?}");
}

#[test]
fn imported_routine_and_enum_variant_short_names_stay_ambiguous() {
    let routines = interface_of(
        "unit Demo.Routines;
         public function Frame(X: integer): integer;
         begin return X; end function;\nend unit;",
    );
    let signals = interface_of(
        "unit Demo.Signals;
         public type Signal = enum Frame(Milliseconds: integer); Other; end enum;\nend unit;",
    );
    let messages = error_messages(
        "unit Demo.Mixed;
         uses Demo.Routines, Demo.Signals;
         public function Run(): integer;
         begin return Frame(2); end function;\nend unit;",
        &[routines, signals],
    );
    assert_eq!(messages.len(), 1, "{messages:#?}");
    assert!(
        messages[0].contains("Ambiguous imported symbol `Frame`"),
        "{messages:#?}"
    );
}
