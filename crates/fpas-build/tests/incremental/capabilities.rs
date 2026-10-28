//! Imported callable results stay unknown in fresh and reused unit interfaces.

use super::{Fixture, assert_output, fs, write};

#[test]
fn cached_factory_results_preserve_dynamic_transfer_checks() {
    for binding in ["const", "var"] {
        let fixture = Fixture::create();
        write(
            &fixture.base,
            &format!(
                "unit Demo.Base;
                public function Make(): function(): integer;
                begin
                  {binding} Value: integer := 42;
                  return function(): integer begin return Value; end function;
                end function; end unit;"
            ),
        );
        write(
            &fixture.root.join("src/consumer.fpas"),
            "unit Demo.Consumer;
            uses Demo.Base as Base, Std.Tasks as Tasks;
            public function Run(): integer;
            begin scope
              const Work := Base.Make();
              const Job := go Work();
              return Tasks.Wait(Job);
            end scope; end function; end unit;",
        );
        for warm in [false, true] {
            let built = fixture
                .build()
                .expect("unknown imported result can compile");
            if warm {
                assert_eq!(built.counters().compiled, 0);
                assert_eq!(built.counters().parsed, 0);
                assert_eq!(built.counters().sidecar_reused, 2);
            }
            if binding == "const" {
                assert_output(built, "42");
            } else {
                let error = fpas_runtime::execution::Vm::new(built.executable)
                    .run()
                    .expect_err("cached interface must preserve runtime transfer guard");
                assert!(error.message.contains("task-bound"), "{error:?}");
            }
        }
        fs::remove_dir_all(&fixture.root).ok();
    }
}
