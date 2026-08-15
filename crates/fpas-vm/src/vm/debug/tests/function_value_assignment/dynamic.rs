//! Dynamic source and destination stay rejected before any function commit.

use super::*;

#[test]
fn dynamic_source_and_destination_preserve_values_and_generation() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_functions(&mut session);
    let locals = scope_reference(&mut session, "Locals");
    let generation = locals;
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let before_current = named(
        &session.variables(locals, 0, 30).expect("before").items,
        "Current",
    )
    .value
    .clone();
    let before_loose = named(
        &session
            .variables(locals, 0, 30)
            .expect("before loose")
            .items,
        "Loose",
    )
    .value
    .clone();

    let source = session
        .set_expression(&root("Current"), &name("Loose"), Some(frame))
        .expect_err("Dynamic source");
    assert_eq!(source.kind, DebugErrorKind::VariableValueType);
    assert!(
        source.message.contains("Dynamic source") || source.hint.contains("Dynamic"),
        "{source:?}"
    );

    let destination = session
        .set_expression(&root("Loose"), &name("Backup"), Some(frame))
        .expect_err("Dynamic destination");
    assert_eq!(destination.kind, DebugErrorKind::VariableValueType);
    assert!(
        destination.message.contains("Dynamic destination")
            || destination
                .message
                .contains("dynamic assignment rejects live or opaque runtime values"),
        "{destination:?}"
    );

    let preserved = session
        .variables(generation, 0, 30)
        .expect("generation survives Dynamic rejection")
        .items;
    assert_eq!(named(&preserved, "Current").value, before_current);
    assert_eq!(named(&preserved, "Loose").value, before_loose);
}
