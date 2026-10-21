//! Channel publication rejects local graphs before changing FIFO state.

use super::*;
use fpas_bytecode::{MemoryDomain, ReferenceRoot, VariableReference};

#[test]
fn every_registry_send_rejects_local_payloads_without_notifying_or_queueing() {
    let registry = ChannelRegistry::new();
    let handle = registry.create(2).expect("channel");
    let domain = MemoryDomain::new();
    let cell = domain.new_cell(Value::Integer(3)).expect("local cell");
    let (program, errors) = fpas_parser::parse("program Transfer; begin end.");
    assert!(errors.is_empty());
    let executable = fpas_compiler::compile(&program).expect("image");
    let target = executable
        .function_target(executable.executable().entry)
        .expect("target");
    let payloads = [
        Value::Cell(cell.clone()),
        Value::option_some(Value::Array(vec![Value::Cell(cell.clone())].into())),
        Value::task_owned_function(target, Vec::new(), domain.id().clone()),
        Value::Reference(Arc::new(VariableReference {
            root: ReferenceRoot::Cell(cell),
            path: Vec::new(),
        })),
    ];
    for value in payloads {
        for wait_for in [None, Some(Duration::ZERO)] {
            assert!(matches!(
                registry.send(handle, Value::Integer(17), false, None),
                Ok(SendState::Sent)
            ));
            let signal = WakeSignal::new();
            let _subscription = registry
                .subscribe_receive(handle, &signal)
                .expect("subscribe");
            assert!(
                registry
                    .send(handle, value.clone(), false, wait_for)
                    .is_err()
            );
            assert!(
                !signal.wait(Duration::ZERO),
                "rejection must not publish readiness"
            );
            assert!(matches!(
                registry.receive(handle, false, None),
                Ok(ReceiveState::Received(Value::Integer(17)))
            ));
            assert!(matches!(
                registry.receive(handle, false, None),
                Ok(ReceiveState::Pending)
            ));
        }
    }
}

#[test]
fn immutable_function_payloads_keep_fifo_and_shared_capture_identity() {
    let registry = ChannelRegistry::new();
    let handle = registry.create(1).expect("channel");
    let (program, errors) = fpas_parser::parse("program Transfer; begin end.");
    assert!(errors.is_empty());
    let executable = fpas_compiler::compile(&program).expect("image");
    let target = executable
        .function_target(executable.executable().entry)
        .expect("target");
    let value = Value::option_some(Value::function(
        target,
        vec![Value::Array(vec![Value::Integer(42)].into())],
    ));
    assert!(matches!(
        registry.send(handle, value.clone(), false, None),
        Ok(SendState::Sent)
    ));
    let ReceiveState::Received(received) = registry.receive(handle, false, None).expect("receive")
    else {
        panic!("transferable callable expected");
    };
    assert_eq!(received, value);
    assert!(matches!(
        registry.receive(handle, false, None),
        Ok(ReceiveState::Pending)
    ));
}

#[test]
fn rejecting_deep_owned_payloads_releases_them_on_a_small_stack() {
    let (program, errors) = fpas_parser::parse("program Transfer; begin end.");
    assert!(errors.is_empty());
    let executable = fpas_compiler::compile(&program).expect("image");
    let target = executable
        .function_target(executable.executable().entry)
        .expect("target");
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(move || {
            let registry = ChannelRegistry::new();
            let handle = registry.create(1).expect("channel");
            let domain = MemoryDomain::new();
            for function_captures in [false, true] {
                let cell = domain.new_cell(Value::Integer(3)).expect("local cell");
                let weak = cell.downgrade();
                let mut payload = Value::Cell(cell);
                for _ in 0..30_000 {
                    payload = if function_captures {
                        Value::function(target.clone(), vec![payload])
                    } else {
                        Value::option_some(payload)
                    };
                }
                assert!(matches!(
                    registry.send(handle, Value::Integer(17), false, None),
                    Ok(SendState::Sent)
                ));
                assert!(registry.send(handle, payload, false, None).is_err());
                assert!(weak.upgrade().is_none(), "rejected payload is released");
                assert!(matches!(
                    registry.receive(handle, false, None),
                    Ok(ReceiveState::Received(Value::Integer(17)))
                ));
                assert!(matches!(
                    registry.receive(handle, false, None),
                    Ok(ReceiveState::Pending)
                ));
            }
        })
        .expect("spawn small-stack rejection test")
        .join()
        .expect("deep rejection must not overflow");
}
