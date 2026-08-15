//! Successful capturing named-routine assignment from compiled metadata.

use super::*;

#[test]
fn unique_simple_and_qualified_nested_names_materialize_from_the_owner_frame() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let frame = run_to(&mut session, "var MakeStop: integer := 0;");
    let updated = session
        .set_expression(&root("Current"), &name("AddBase"), Some(frame))
        .expect("simple nested name");
    assert_eq!(updated.value, "<function makeadder.addbase>");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    assert_eq!(rendered(&mut session, call("Current", 1), frame), "11");

    let mut qualified = DebugSession::new(compile_fixture()).expect("qualified session");
    let frame = run_to(&mut qualified, "var MakeStop: integer := 0;");
    let updated = qualified
        .set_expression(
            &root("Current"),
            &DebugExpression::Field {
                base: Box::new(name("MakeAdder")),
                name: "AddBase".to_string(),
            },
            Some(frame),
        )
        .expect("qualified nested name");
    assert_eq!(updated.value, "<function makeadder.addbase>");
    let frame = qualified.stack(0, 1).expect("fresh").items[0].id;
    assert_eq!(rendered(&mut qualified, call("Current", 1), frame), "11");
}

#[test]
fn multiple_immutable_captures_preserve_closure_abi_order() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let frame = run_to(&mut session, "var CombineStop: integer := 0;");
    session
        .set_expression(&root("Current"), &name("AddBoth"), Some(frame))
        .expect("two captures");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    assert_eq!(rendered(&mut session, call("Current", 1), frame), "35");
}

#[test]
fn selected_recursive_activation_supplies_its_own_captured_values() {
    let mut inner = DebugSession::new(compile_fixture()).expect("inner walk");
    let frame = run_to_hit(&mut inner, "var WalkStop: integer := 0;", 3);
    inner
        .set_expression(&root("Current"), &name("AddAcc"), Some(frame))
        .expect("innermost Acc");
    let frame = inner.stack(0, 1).expect("fresh inner").items[0].id;
    assert_eq!(rendered(&mut inner, call("Current", 0), frame), "21");

    let mut outer = DebugSession::new(compile_fixture()).expect("outer walk");
    let _ = run_to_hit(&mut outer, "var WalkStop: integer := 0;", 3);
    let outer_frame = outer.stack(0, 8).expect("walk stack").items[2].id;
    outer
        .set_expression(&root("Current"), &name("AddAcc"), Some(outer_frame))
        .expect("outer Acc");
    let selected = outer.stack(0, 8).expect("fresh outer").items[2].id;
    assert_eq!(rendered(&mut outer, call("Current", 0), selected), "1");
}

#[test]
fn same_name_shadow_cannot_replace_the_exact_owner_binding() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let frame = run_to(&mut session, "var ShadowStop: integer := 0;");
    session
        .set_expression(&root("Current"), &name("AddOffset"), Some(frame))
        .expect("parameter Offset");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    assert_eq!(rendered(&mut session, call("Current", 1), frame), "8");
}

#[test]
fn initialized_late_local_can_be_captured_after_its_assignment() {
    let mut session = DebugSession::new(compile_fixture()).expect("debug session");
    let frame = run_to(&mut session, "var LateReady: integer := 0;");
    session
        .set_expression(&root("Current"), &name("AddLate"), Some(frame))
        .expect("initialized Offset");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    assert_eq!(rendered(&mut session, call("Current", 1), frame), "8");
}
