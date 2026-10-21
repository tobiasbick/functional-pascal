use super::*;

#[test]
fn if_and_case_join_independent_arm_entries_in_both_orders() {
    for (first, second) in [("Local", "Free"), ("Free", "Local")] {
        for decision in [
            format!("if Flag then Chosen := {first}; else Chosen := {second}; end if;"),
            format!(
                "case Flag of when true: Chosen := {first}; when false: Chosen := {second}; end case;"
            ),
            format!("case Count of when 1: Chosen := {first}; else Chosen := {second}; end case;"),
        ] {
            rejects_local("", &format!("{decision} discard go Chosen();"));
        }
    }
}

#[test]
fn an_arm_does_not_observe_assignments_from_an_alternative_arm() {
    for decision in [
        "if Flag then Chosen := Local; else discard go Chosen(); end if;",
        "case Flag of when true: Chosen := Local; when false: discard go Chosen(); end case;",
        "case Count of when 1: Chosen := Local; else discard go Chosen(); end case;",
    ] {
        check_ok(&program("", decision));
    }
}

#[test]
fn omitted_else_preserves_entry_and_whole_overwrite_after_join_clears_it() {
    rejects_local(
        "",
        "Chosen := Local; if Flag then Chosen := Free; end if; discard go Chosen();",
    );
    check_ok(&program(
        "",
        "if Flag then Chosen := Local; end if; Chosen := Free; discard go Chosen();",
    ));
    check_ok(&program(
        "",
        "Chosen := Local; if Flag then Chosen := Free; else Chosen := Free; end if; discard go Chosen();",
    ));
    check_ok(&program(
        "",
        "Chosen := Local; case Count of when 1: Chosen := Free; else Chosen := Free; end case; discard go Chosen();",
    ));
}

#[test]
fn nested_shadowing_does_not_change_the_outer_binding() {
    check_ok(&program(
        "",
        "if Flag then var Chosen: function(): integer := Local; discard Chosen; else var Chosen: function(): integer := Local; discard Chosen; end if; discard go Chosen();",
    ));
    rejects_local(
        "",
        "const Alias: function(): integer := Local; if Flag then var Alias: function(): integer := Free; Chosen := Alias; else Chosen := Alias; end if; discard go Chosen();",
    );
}

#[test]
fn exhaustive_cases_do_not_keep_an_impossible_unmatched_exit() {
    check_ok(&program(
        "",
        "Chosen := Local; case Flag of when true: Chosen := Free; when false: Chosen := Free; end case; discard go Chosen();",
    ));
    check_ok(&program(
        "",
        "Chosen := Local; case Some(1) of when Some(_): Chosen := Free; when None: Chosen := Free; end case; discard go Chosen();",
    ));
}

#[test]
fn abrupt_arms_do_not_contribute_to_the_normal_join() {
    check_ok(&program(
        "",
        "if Flag then Chosen := Local; return; else Chosen := Free; end if; discard go Chosen();",
    ));
    check_ok(&program(
        "",
        "case Flag of when true: Chosen := Local; return; when false: Chosen := Free; end case; discard go Chosen();",
    ));
}
