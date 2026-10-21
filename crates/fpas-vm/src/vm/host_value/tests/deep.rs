use super::*;

#[test]
fn deep_rejection_extraction_selection_and_export_drop_use_bounded_stack() {
    let image = empty_image();
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(move || {
            let context = HostValues::new(MemoryDomain::new());
            let mut rejected = Value::OpaqueHandle(1);
            let mut pure = Value::Integer(42);
            for _ in 0..20_000 {
                rejected = Value::option_some(rejected);
                pure = Value::result_ok(pure);
            }
            assert_eq!(
                context
                    .admit(vec![rejected.into()], &image)
                    .expect_err("deep hidden resource")
                    .code,
                RUNTIME_INVALID_TASK
            );
            let value = exported(&context, pure);
            let child = value
                .child(HostChild::Payload)
                .expect("deep child")
                .expect("payload");
            let copy = value.try_to_data().expect("deep pure extraction");
            release_values(vec![copy]);
            let imported = context
                .admit(vec![child.into()], &image)
                .expect("deep import");
            release_values(imported.0);
            drop(imported.1);
            drop(value);
        })
        .expect("bounded-stack host thread")
        .join()
        .expect("deep host operations");
}
