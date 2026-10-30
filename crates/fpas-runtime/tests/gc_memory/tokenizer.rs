//! Immutable tokenizer tables belong to the VM and leave no process-global cache behind.

use crate::{
    measurement::completed_threads,
    workloads::{image, schedule},
};
use fpas_runtime::{CollectionSchedule, execution::Vm};

pub fn teardown() {
    // regex-automata retains only a process-global usize for its default CPU count.
    // Initialize that metadata directly, without a regex, tokenizer or FPAS session.
    // First/repeated VM runs still have no byte allowance or tokenizer warm-up.
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
    assert!(regex_automata::meta::Config::new().get_pool_capacity() > 0);
    assert_eq!(crate::measurement::Footprint::now(), after);
    let executable = image(include_str!(
        "../../../../tests/stdlib/tokenizer/tasks_test.fpas"
    ));
    for config in [CollectionSchedule::default(), schedule()] {
        completed_threads("tokenizer tables and callback captures", 4, || {
            Vm::new(executable.clone())
                .with_collection_schedule(config)
                .run()
                .unwrap();
        });
    }
}
