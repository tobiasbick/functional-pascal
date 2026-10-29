//! Selecting must not wait for concurrent launch or alias cleanup locks.

use super::*;

#[test]
fn busy_registry_or_process_is_pending_without_running_the_operation() {
    let processes = PipedProcesses::default();
    let mut registry = processes.registry.lock().unwrap();
    assert!(matches!(
        processes.try_with::<()>(1, |_| panic!("registry busy")),
        Ok(None)
    ));
    let entry = Arc::new(Mutex::new(None));
    registry.entries.insert(1, entry.clone());
    drop(registry);
    let process = entry.lock().unwrap();
    assert!(matches!(
        processes.try_with::<()>(1, |_| panic!("alias busy")),
        Ok(None)
    ));
    drop(process);
    assert!(matches!(
        processes.try_with::<()>(1, |_| panic!("closed entry")),
        Err(Failure {
            kind: Kind::Argument("Process", _),
            ..
        })
    ));
}
