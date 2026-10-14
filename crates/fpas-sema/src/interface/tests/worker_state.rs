//! Worker dependency enforcement survives independently encoded unit interfaces.

use super::{analyze_unit, parse_unit};
use fpas_diagnostics::codes::SEMA_FOREIGN_TASK_STATE;

#[test]
fn imported_helper_and_group_callback_dependencies_survive_interface_roundtrip() {
    let dependency = parse_unit(
        "unit Demo.Workers; uses Std.Tasks as Tasks;\n        var Counter: integer := 1;\n        public function Read(): integer; begin return Counter; end function;\n        public procedure Start(Group: Tasks.TaskGroup; Work: function(Token: Tasks.CancellationToken): integer);\n        begin discard Tasks.StartTaskInGroup(Group, Work); end procedure;\n        end unit;",
    );
    let analysis = analyze_unit(&dependency, &[]).unwrap();
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let original = analysis.interface.unwrap();
    let bytes = fpas_unit::interface::encode_interface(&original).unwrap();
    let decoded = fpas_unit::interface::decode_interface(&bytes).unwrap();
    for interface in [original, decoded] {
        for body in [
            "scope discard go Workers.Read(); end scope;",
            "const Group := Tasks.CreateTaskGroup(); Workers.Start(Group, Work);",
        ] {
            let consumer = parse_unit(&format!(
                "unit Demo.Consumer; uses Std.Tasks as Tasks, Demo.Workers as Workers;\n                function Work(Token: Tasks.CancellationToken): integer; begin return Workers.Read(); end function;\n                public procedure Run(); begin {body} end procedure; end unit;"
            ));
            let analysis = analyze_unit(&consumer, std::slice::from_ref(&interface)).unwrap();
            assert!(
                analysis
                    .metadata
                    .errors
                    .iter()
                    .any(|error| error.code == SEMA_FOREIGN_TASK_STATE),
                "{body}: {:?}",
                analysis.metadata.errors
            );
        }
    }
}
