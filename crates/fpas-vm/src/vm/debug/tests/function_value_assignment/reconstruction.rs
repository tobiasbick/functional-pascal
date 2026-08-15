//! Synthetic callable children stay inspection-only; complete replacement remains the write model.

use super::*;

fn call(name: &str, value: i64) -> DebugExpression {
    DebugExpression::Call {
        callee: Box::new(super::name(name)),
        arguments: vec![DebugExpression::Integer(value)],
    }
}

fn bound_add() -> DebugExpression {
    DebugExpression::Field {
        base: Box::new(name("Box")),
        name: "Add".to_string(),
    }
}

#[test]
fn synthetic_capture_and_receiver_children_are_not_assignable() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let locals = scope_reference(&mut session, "Locals");
    let generation = locals;
    let locals_page = session.variables(locals, 0, 30).expect("locals");
    let captured = named(&locals_page.items, "Captured");
    assert_ne!(
        captured.variables_reference, 0,
        "captured closures expose inspection children"
    );
    let capture_handle = captured.variables_reference;
    let children = session
        .variables(capture_handle, 0, 8)
        .expect("capture children");
    assert!(
        children
            .items
            .iter()
            .any(|child| child.name == "capture[0]"),
        "{children:?}"
    );

    let capture = session
        .set_variable(capture_handle, "capture[0]", &DebugExpression::Integer(99))
        .expect_err("synthetic capture child");
    assert_eq!(capture.kind, DebugErrorKind::VariablePathUnsupported);
    assert!(capture.message.contains("not assignable"), "{capture:?}");

    let preserved = session
        .variables(generation, 0, 30)
        .expect("generation survives capture-child rejection")
        .items;
    assert_eq!(named(&preserved, "Captured").value, "<function adder>");
    let frame = session.stack(0, 1).expect("frame").items[0].id;
    assert_eq!(
        session
            .evaluate(&call("Captured", 1), Some(frame))
            .expect("original capture graph")
            .value,
        "11"
    );

    session
        .set_expression(&root("Current"), &bound_add(), Some(frame))
        .expect("complete bound-method replacement remains supported");
    let locals = scope_reference(&mut session, "Locals");
    let generation = locals;
    let locals_page = session.variables(locals, 0, 30).expect("bound local");
    let current = named(&locals_page.items, "Current");
    assert_ne!(current.variables_reference, 0);
    let receiver_handle = current.variables_reference;
    let children = session
        .variables(receiver_handle, 0, 8)
        .expect("receiver children");
    assert!(
        children.items.iter().any(|child| child.name == "receiver"),
        "{children:?}"
    );

    let receiver = session
        .set_variable(receiver_handle, "receiver", &name("Box"))
        .expect_err("synthetic receiver child");
    assert_eq!(receiver.kind, DebugErrorKind::VariablePathUnsupported);
    assert!(receiver.message.contains("not assignable"), "{receiver:?}");

    let preserved = session
        .variables(generation, 0, 30)
        .expect("generation survives receiver-child rejection")
        .items;
    assert_eq!(named(&preserved, "Current").value, "<function Holder.Add>");
    let frame = session.stack(0, 1).expect("bound frame").items[0].id;
    assert_eq!(
        session
            .evaluate(&call("Current", 5), Some(frame))
            .expect("complete bound replacement still invokes")
            .value,
        "8"
    );
}
