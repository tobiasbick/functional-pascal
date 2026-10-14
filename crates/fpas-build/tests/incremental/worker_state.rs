//! Cold/warm builds enforce worker callback dependencies from compiled units.

use super::{Fixture, assert_output, fs, write};

#[test]
fn cached_unit_worker_summaries_reject_foreign_globals_through_imported_starters() {
    let fixture = Fixture::create();
    write(
        &fixture.base,
        "unit Demo.Base; uses Std.Tasks as Tasks;\n        var Counter: integer := 1;\n        public function Read(): integer; begin return Counter; end function;\n        public function Start(Group: Tasks.TaskGroup; Work: function(Token: Tasks.CancellationToken): integer): task of integer;\n        begin return Tasks.StartTaskInGroup(Group, Work); end function; end unit;",
    );
    let consumer = fixture.root.join("src/consumer.fpas");
    let source = |body: &str| {
        format!(
            "unit Demo.Consumer; uses Std.Tasks as Tasks, Demo.Base as API;\n        function Work(Token: Tasks.CancellationToken): integer; begin {body} end function;\n        public function Run(): integer; begin\n        const Group := Tasks.CreateTaskGroup(); const Job := API.Start(Group, Work);\n        const Value := Tasks.Wait(Job); discard Tasks.CloseTaskGroup(Group); return Value;\n        end function; end unit;"
        )
    };
    write(&consumer, &source("return 7;"));
    let cold = fixture.build().expect("cold safe workers");
    assert_eq!(cold.counters().compiled, 2);
    assert_output(cold, "7");
    let warm = fixture.build().expect("warm safe workers");
    assert_eq!(warm.counters().sidecar_reused, 2);
    assert_eq!(warm.counters().parsed, 0);
    assert_output(warm, "7");
    write(&consumer, &source("return API.Read();"));
    for _ in 0..2 {
        let failure = match fixture.build() {
            Err(error) => error,
            Ok(_) => panic!("imported worker starter authorized a mutable global read"),
        };
        assert!(
            failure
                .diagnostics()
                .iter()
                .any(|record| record.diagnostic.code
                    == fpas_diagnostics::codes::SEMA_FOREIGN_TASK_STATE),
            "{failure:#?}"
        );
    }
    fs::remove_dir_all(&fixture.root).ok();
}
