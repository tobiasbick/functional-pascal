//! Compiled engines and their scratch pools leave no retained bytes after VM teardown.
use crate::{
    measurement::completed_threads,
    workloads::{image, schedule},
};
use fpas_runtime::{CollectionSchedule, execution::Vm};

pub fn teardown() {
    // Account only for the regex engine's one process-global CPU-count word.
    // No pattern or FPAS session is warmed up before allocation measurement.
    let before = crate::measurement::Footprint::now();
    assert!(regex_automata::meta::Config::new().get_pool_capacity() > 0);
    let after = crate::measurement::Footprint::now();
    assert!(
        after == before
            || after
                == crate::measurement::Footprint {
                    bytes: before.bytes + std::mem::size_of::<usize>() as i128,
                    allocations: before.allocations + 1,
                }
    );
    let tasks = image(include_str!(
        "../../../../tests/stdlib/regex/tasks_test.fpas"
    ));
    let failing = image(
        r#"program RegexFailure;
uses Std.Text.Regex as Regex;
begin
  const Pattern := Regex.Compile('(\w+)', Regex.DefaultOptions()).Unwrap();
  var Cycle: function(): integer := function(): integer begin return 0; end function;
  Cycle := function(): integer begin
    if Regex.IsMatch(Pattern, 'abc').Unwrap() then return 1; end if;
    return Cycle();
  end function;
  discard Cycle();
  panic('expected regex cleanup failure');
end."#,
    );
    for config in [CollectionSchedule::default(), schedule()] {
        completed_threads("compiled regex task transfer and callbacks", 4, || {
            Vm::new(tasks.clone())
                .with_collection_schedule(config)
                .run()
                .unwrap();
        });
        completed_threads("compiled regex captured cycle and root failure", 4, || {
            assert!(
                Vm::new(failing.clone())
                    .with_collection_schedule(config)
                    .run()
                    .is_err()
            );
        });
    }
}
