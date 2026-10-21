use super::*;

#[test]
fn possibly_empty_loops_keep_entry_capabilities() {
    for body in [
        "while Flag do Chosen := Free; end while;",
        "for I: integer := 1 to Count do Chosen := Free; end for;",
        "const Items: array of integer := []; for I: integer in Items do Chosen := Free; end for;",
    ] {
        rejects_local("", &format!("Chosen := Local; {body} discard go Chosen();"));
    }
}

#[test]
fn loop_backedges_recheck_earlier_transfers() {
    for body in [
        "while Flag do discard go Chosen(); Chosen := Local; end while;",
        "for I: integer := 1 to Count do discard go Chosen(); Chosen := Local; end for;",
        "for I: integer in [1, 2] do discard go Chosen(); Chosen := Local; end for;",
        "repeat discard go Chosen(); Chosen := Local; until Flag;",
    ] {
        let errors = check_errors(&program("", body));
        assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TASK_BOUND_CALLABLE, "{body}");
    }
}

#[test]
fn repeat_runs_once_and_full_overwrites_can_make_its_exit_safe() {
    check_ok(&program(
        "",
        "Chosen := Local; repeat Chosen := Free; until Flag; discard go Chosen();",
    ));
    check_ok(&program(
        "",
        "while Flag do Chosen := Free; discard go Chosen(); Chosen := Local; end while;",
    ));
}

#[test]
fn break_and_continue_keep_the_state_at_the_actual_control_transfer() {
    rejects_local(
        "",
        "repeat Chosen := Local; break; Chosen := Free; until Flag; discard go Chosen();",
    );
    rejects_local(
        "",
        "while Flag do discard go Chosen(); Chosen := Local; continue; Chosen := Free; end while;",
    );
    rejects_local(
        "",
        "repeat Chosen := Local; continue; Chosen := Free; until Flag; discard go Chosen();",
    );
}

#[test]
fn loop_local_shadowing_does_not_taint_an_outer_callable() {
    check_ok(&program(
        "",
        "while Flag do const Chosen: function(): integer := Local; discard Chosen; break; end while; discard go Chosen();",
    ));
}

#[test]
fn unconditional_break_and_return_do_not_create_a_second_iteration() {
    for body in [
        "while Flag do discard go Chosen(); Chosen := Local; break; end while;",
        "for I: integer := 1 to Count do discard go Chosen(); Chosen := Local; break; end for;",
        "for I: integer in [1, 2] do discard go Chosen(); Chosen := Local; return; end for;",
        "repeat discard go Chosen(); Chosen := Local; return; until Flag;",
        "while Flag do discard go Chosen(); Chosen := Local; if Flag then break; else return; end if; end while;",
        "while Flag do discard go Chosen(); Chosen := Local; case Flag of when true: break; when false: return; end case; end while;",
    ] {
        check_ok(&program("", body));
    }
}

#[test]
fn unreachable_tail_writes_and_continues_do_not_add_backedges() {
    check_ok(&program(
        "",
        "while Flag do discard go Chosen(); break; Chosen := Local; continue; end while; discard go Chosen();",
    ));
    check_ok(&program(
        "",
        "while Flag do discard go Chosen(); return; Chosen := Local; continue; end while; discard go Chosen();",
    ));
    check_ok(&program(
        "",
        "while Flag do while Flag do Chosen := Local; return; end while; discard go Chosen(); break; end while;",
    ));
    rejects_local(
        "",
        "while Flag do while Flag do Chosen := Local; break; end while; discard go Chosen(); break; end while;",
    );
}
