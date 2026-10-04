//! Imported source declarations require their declared alias.

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
fn source_and_std_names_require_aliases_even_with_different_arity() {
    let sender = interface_of(
        r#"unit Demo.Sender;
         public function Send(A: integer; B: integer; C: integer): integer;
         begin return A + B + C; end function;
end unit;
"#,
    );
    let interfaces = [sender];

    let short = error_messages(
        r#"unit Demo.Short;
uses Demo.Sender as Sender; uses Std.Tasks as Tasks;
         public function Run(): integer;
         begin return Send(1, 2, 3); end function;
end unit;
"#,
        &interfaces,
    );
    assert_eq!(short.len(), 1, "{short:#?}");
    assert!(
        short[0].contains("Unknown function or procedure `Send`"),
        "{short:#?}"
    );

    let qualified = error_messages(
        r#"unit Demo.Qualified;
uses Demo.Sender as Sender; uses Std.Tasks as Tasks;
         public function Run(): integer;
         begin return Sender.Send(1, 2, 3); end function;
end unit;
"#,
        &interfaces,
    );
    assert!(qualified.is_empty(), "{qualified:#?}");

    let without_std = error_messages(
        r#"unit Demo.WithoutStd;
uses Demo.Sender as Sender;
         public function Run(): integer;
         begin return Send(1, 2, 3); end function;
end unit;
"#,
        &interfaces,
    );
    assert_eq!(without_std.len(), 1, "{without_std:#?}");
}

#[test]
fn qualified_std_use_does_not_open_source_names() {
    let first = interface_of(
        r#"unit Demo.First;
         public function Value(): integer;
         begin return 1; end function;
end unit;
"#,
    );
    let second = interface_of(
        r#"unit Demo.Second;
         public function Value(): integer;
         begin return 2; end function;
end unit;
"#,
    );
    let messages = error_messages(
        r#"unit Demo.Lazy;
uses Demo.First as First; uses Demo.Second as Second; uses Std.Math as Math;
         public function Run(): integer;
         begin return Math.Abs(-1) + Value(); end function;
end unit;
"#,
        &[first, second],
    );
    assert_eq!(messages.len(), 1, "{messages:#?}");
    assert!(
        messages[0].contains("Unknown function or procedure `Value`"),
        "{messages:#?}"
    );
}

#[test]
fn alias_paths_distinguish_imported_types_and_variants() {
    let frames = interface_of(
        r#"unit Demo.Frames;
           public type Frame = record public X: integer; end record;
end unit;
"#,
    );
    let signals = interface_of(
        r#"unit Demo.Signals;
           public type Signal = enum Frame(Milliseconds: integer); Other; end enum;
end unit;
"#,
    );
    let interfaces = [frames, signals];

    // Types and variants use their unit alias.
    let messages = error_messages(
        r#"unit Demo.Mixed;
uses Demo.Frames as Frames; uses Demo.Signals as Signals;
         public function Run(): integer;
         begin
           var F: Frames.Frame := Frames.Frame(X := 1);
           var S: Signals.Signal := Signals.Signal.Frame(2);
           return F.X;
         end function;
end unit;
"#,
        &interfaces,
    );
    assert!(messages.is_empty(), "{messages:#?}");

    let variant_short = error_messages(
        r#"unit Demo.VariantShort;
uses Demo.Frames as Frames; uses Demo.Signals as Signals;
         public function Run(): Signals.Signal;
         begin return Frame(2); end function;
end unit;
"#,
        &interfaces,
    );
    assert_eq!(variant_short.len(), 1, "{variant_short:#?}");
}

#[test]
fn imported_routines_and_variants_do_not_open_short_names() {
    let routines = interface_of(
        r#"unit Demo.Routines;
         public function Frame(X: integer): integer;
         begin return X; end function;
end unit;
"#,
    );
    let signals = interface_of(
        r#"unit Demo.Signals;
           public type Signal = enum Frame(Milliseconds: integer); Other; end enum;
end unit;
"#,
    );
    let messages = error_messages(
        r#"unit Demo.Mixed;
uses Demo.Routines as Routines; uses Demo.Signals as Signals;
         public function Run(): integer;
         begin return Frame(2); end function;
end unit;
"#,
        &[routines, signals],
    );
    assert_eq!(messages.len(), 1, "{messages:#?}");
    assert!(
        messages[0].contains("Unknown function or procedure `Frame`"),
        "{messages:#?}"
    );
}
