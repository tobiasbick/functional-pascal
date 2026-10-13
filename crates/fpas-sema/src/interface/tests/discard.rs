//! Discard is ordinary value disposal across encoded unit interfaces.

use super::{analyze_unit, parse_unit};

#[test]
fn imported_task_capturing_factories_and_bound_methods_need_no_discard_proof() {
    let dependency = parse_unit(
        "unit Demo.Callbacks; uses Std.Tasks;
        public function Make(Job: task of integer): function(): integer;
        begin return function(): integer begin return Wait(Job); end function; end function;
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
    let bytes = fpas_unit::interface::encode_interface(&analysis.interface.unwrap()).unwrap();
    let interface = fpas_unit::interface::decode_interface(&bytes).unwrap();
    let consumer = parse_unit(
        "unit Demo.Consumer; uses Std.Tasks, Demo.Callbacks;
        public procedure Run(Job: task of integer); begin
        discard Make(Job); discard Demo.Callbacks.Make(Job); discard [Job];
        const Value: Box := Box(Value := 1);
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
