//! Bound native receivers can close cycles through their captured array elements.

use crate::{
    measurement::completed_threads,
    workloads::{image, schedule},
};
use fpas_runtime::{CollectionSchedule, execution::Vm};

pub fn teardown() {
    let source = include_str!("../../../../tests/stdlib/fluent/native_callable_gc_test.fpas");
    for failing in [false, true] {
        let source = if failing {
            let (body, _) = source.rsplit_once("end.").unwrap();
            format!("{body} panic('expected native callable failure'); end.")
        } else {
            source.to_owned()
        };
        let executable = image(&source);
        for config in [CollectionSchedule::default(), schedule()] {
            completed_threads("native callable receiver cycles", 4, || {
                let result = Vm::new(executable.clone())
                    .with_collection_schedule(config)
                    .run();
                assert_eq!(result.is_err(), failing);
            });
        }
    }
}
