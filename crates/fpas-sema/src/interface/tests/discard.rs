//! Discard is ordinary value disposal across encoded unit interfaces.

use super::{analyze_unit, parse_unit};

#[test]
fn imported_task_capturing_factories_and_bound_methods_need_no_discard_proof() {
    let dependency = parse_unit(
        "unit Demo.Callbacks; uses Std.Tasks as Tasks;\n        public function Make(Job: task of integer): function(): integer;\n        begin return function(): integer begin return Tasks.Wait(Job); end function; end function;\n        public type Box = record public Value: integer;\n        public function Make(Self: Box): function(): integer;\n        begin return function(): integer begin return Self.Value; end function; end function;\n        end record; end unit;",
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
        "unit Demo.Consumer; uses Std.Tasks as Tasks, Demo.Callbacks as Callbacks;\n        public procedure Run(Job: task of integer); begin\n        discard Callbacks.Make(Job); discard Callbacks.Make(Job); discard [Job];\n        const Value: Callbacks.Box := Callbacks.Box(Value := 1);\n        discard Value.Make();\n        const Bound: function(): function(): integer := Value.Make;\n        discard Bound();\n        end procedure; end unit;",
    );
    let analysis = analyze_unit(&consumer, &[interface]).unwrap();
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
}
