use super::{analyze_unit, parse_unit};
use fpas_diagnostics::codes::SEMA_UNSAFE_DISCARD;

#[test]
fn task_captures_change_the_persistent_factory_contract() {
    let unit = |body: &str| {
        parse_unit(&format!(
            "unit Demo.Contract; uses Std.Tasks;
      public function Make(Job: task of integer): function(): integer;
      begin return function(): integer begin {body} end function; end function;
      end unit;"
        ))
    };
    let safe = analyze_unit(&unit("return 1;"), &[])
        .unwrap()
        .interface
        .unwrap();
    let unsafe_value = analyze_unit(&unit("return Wait(Job);"), &[])
        .unwrap()
        .interface
        .unwrap();
    assert!(safe.symbols[0].discard.result);
    assert!(!unsafe_value.symbols[0].discard.result);
    assert_ne!(safe.digest().unwrap(), unsafe_value.digest().unwrap());
}

#[test]
fn record_method_result_proofs_survive_unit_interfaces_and_binding() {
    let dependency = parse_unit(
        "unit Demo.Methods;
      public type Box = record public Value: integer;
      public function Make(Self: Box): function(): integer;
      begin return function(): integer begin return Self.Value; end function; end function;
      end record; end unit;",
    );
    let analysis = analyze_unit(&dependency, &[]).unwrap();
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.unwrap();
    let consumer = parse_unit(
        "unit Demo.MethodConsumer; uses Demo.Methods;
      public procedure Run(); begin
      const Value: Box := record Value := 1; end;
      discard Value.Make();
      const Bound: function(): function(): integer := Value.Make;
      discard Bound();
      end procedure; end unit;",
    );
    let analysis = analyze_unit(&consumer, &[interface]).unwrap();
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
}

#[test]
fn closure_factory_proofs_survive_encoded_unit_interfaces() {
    let dependency = parse_unit(
        "unit Demo.Callbacks;
      public function Make(Value: integer): function(): integer;
      begin return function(): integer begin return Value; end function; end function;
      end unit;",
    );
    let analysis = analyze_unit(&dependency, &[]).unwrap();
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.unwrap();
    assert!(interface.symbols[0].discard.result);
    let encoded = fpas_unit::interface::encode_interface(&interface).unwrap();
    let interface = fpas_unit::interface::decode_interface(&encoded).unwrap();
    let consumer = parse_unit(
        "unit Demo.Consumer; uses Demo.Callbacks;
      public procedure Run(); begin discard Make(1); discard Demo.Callbacks.Make(2); end procedure;
      end unit;",
    );
    let analysis = analyze_unit(&consumer, std::slice::from_ref(&interface)).unwrap();
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let mut unknown = interface;
    unknown.symbols[0].discard.result = false;
    let analysis = analyze_unit(&consumer, &[unknown]).unwrap();
    assert_eq!(
        analysis
            .metadata
            .errors
            .iter()
            .filter(|error| error.code == SEMA_UNSAFE_DISCARD)
            .count(),
        2
    );
}
