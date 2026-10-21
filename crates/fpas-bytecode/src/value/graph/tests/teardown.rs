use super::*;

#[test]
fn ordinary_function_drop_releases_deep_capture_and_receiver_chains() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let owner = MemoryDomain::new();
            let cell = owner.new_cell(Value::Unit).unwrap();
            let weak = cell.downgrade();
            let function = target();
            let mut value = Value::Cell(cell);
            for index in 0..30_000 {
                value = if index % 2 == 0 {
                    Value::function(function.clone(), vec![value])
                } else {
                    Value::bound_function(function.clone(), value)
                };
            }
            let shared = value.clone();
            drop(value);
            assert!(weak.upgrade().is_some());
            drop(shared);
            assert!(weak.upgrade().is_none());
            assert_eq!(owner.registered_count(), 0);
        })
        .unwrap()
        .join()
        .unwrap();
}
