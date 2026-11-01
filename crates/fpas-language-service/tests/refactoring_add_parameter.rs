//! Signature propagation is complete, capture-safe and repeatable through ordinary plans.
#![allow(
    clippy::expect_used,
    reason = "fixtures require exact checked outcomes"
)]

use crate::support::{self, TempDirectory};
use fpas_language_service::{
    CancellationToken, ParameterMode, RefactorPreparation, RefactorRecipe, RefactorStatus,
    SourceSelector, SourceSession,
};
use std::path::Path;

fn recipe(
    symbol: &str,
    name: &str,
    ty: &str,
    mode: ParameterMode,
    argument: &str,
) -> RefactorRecipe {
    RefactorRecipe::AddParameter {
        selector: SourceSelector::Symbol {
            name: symbol.into(),
        },
        name: name.into(),
        r#type: ty.into(),
        mode,
        argument: argument.into(),
    }
}
fn prepare(context: &Path, recipe: &RefactorRecipe) -> RefactorPreparation {
    SourceSession::load(context, None, &[], &CancellationToken::new())
        .expect("scope")
        .prepare_refactor(recipe, &CancellationToken::new())
        .expect("prepare")
}
fn fixture(source: &str) -> (TempDirectory, std::path::PathBuf, std::path::PathBuf) {
    let temp = TempDirectory::new("add-parameter");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let path = temp.write("main.fpas", source);
    (temp, manifest, path)
}
fn changed(prepared: &RefactorPreparation) -> &str {
    assert_eq!(prepared.status, RefactorStatus::Prepared, "{prepared:?}");
    &prepared.changes.as_ref().expect("changes").changes[0].replacement
}

#[test]
fn append_preserves_recursion_named_order_comments_unicode_and_line_endings() {
    let source = "program Main;\r\n// Grüße 🦀\r\nconst Base: integer := 7;\r\nfunction Step(N: integer // existing parameter\r\n): integer;\r\nbegin if N > 0 then return Step(N - 1 // existing argument\r\n); end if; return 0; end function;\r\nbegin discard Step(N := 3); discard Step(2); end.\r\n";
    let (_temp, context, path) = fixture(source);
    let request = recipe(
        "Step",
        "Offset",
        "integer // type",
        ParameterMode::Value,
        "Base // argument",
    );
    let prepared = prepare(&context, &request);
    let text = changed(&prepared);
    assert!(text.contains("integer // existing parameter\r\n; Offset: integer // type\r\n)"));
    assert!(text.contains("Step(N := 3, Offset := Base // argument\r\n)"));
    assert!(text.contains("Step(N - 1 // existing argument\r\n, Base // argument\r\n)"));
    assert_eq!(prepared.plan.as_ref().expect("plan").edits.len(), 4);
    assert_eq!(std::fs::read_to_string(&path).expect("unchanged"), source);
    assert_eq!(prepare(&context, &request).plan, prepared.plan);
    std::fs::write(&path, text).expect("apply fixture");
    let repeated = SourceSession::load(&context, None, &[], &CancellationToken::new())
        .expect("after scope")
        .revalidate_refactor(
            prepared.plan.as_ref().expect("plan"),
            &CancellationToken::new(),
        )
        .expect("repeat");
    assert_eq!(repeated.status, RefactorStatus::AlreadyApplied);
}

#[test]
fn imports_nested_routines_record_defaults_and_empty_unused_signatures_are_supported() {
    let temp = TempDirectory::new("parameter-imports");
    let (manifest, main, _) = support::write_program_project(&temp);
    let imported = prepare(
        &manifest,
        &recipe(
            "Demo.Math.Answer",
            "Extra",
            "integer",
            ParameterMode::Value,
            "4",
        ),
    );
    assert_eq!(imported.status, RefactorStatus::Prepared, "{imported:?}");
    assert_eq!(imported.plan.as_ref().expect("plan").edits.len(), 2);
    assert!(
        imported
            .changes
            .as_ref()
            .expect("changes")
            .changes
            .iter()
            .any(
                |c| fpas_language_service::identity_path(&c.path).expect("change path")
                    == fpas_language_service::identity_path(
                        &std::fs::canonicalize(&main).expect("main")
                    )
                    .expect("main path")
                    && c.replacement.contains("Math.Answer(4)")
            )
    );
    for (source, target, count) in [
        (
            "program Main; function Outer(): integer; function Inner(): integer; begin return 2; end function; begin return Inner(); end function; begin discard Outer(); end.",
            "Main.Outer.Inner",
            2,
        ),
        (
            "program Main; function Seed(): integer; begin return 7; end function; type Item = record Value: integer := Seed(); end record; begin const A := Item(); const B := Item(); end.",
            "Seed",
            2,
        ),
        (
            "program Main; procedure Unused(); begin end procedure; begin end.",
            "Unused",
            1,
        ),
    ] {
        let (_temp, context, _) = fixture(source);
        let prepared = prepare(
            &context,
            &recipe(target, "Extra", "integer", ParameterMode::Value, "1"),
        );
        changed(&prepared);
        assert_eq!(prepared.plan.expect("plan").edits.len(), count);
    }
}

#[test]
fn var_arguments_require_writable_disjoint_storage_and_preserve_existing_modes() {
    let source = "program Main; procedure Touch(var Value: integer); begin Value := Value + 1; end procedure; begin var A := 0; var B := 1; Touch(var A); end.";
    let (_temp, context, path) = fixture(source);
    let prepared = prepare(
        &context,
        &recipe("Touch", "Extra", "integer", ParameterMode::Var, "B"),
    );
    assert!(changed(&prepared).contains("Touch(var A, var B)"));
    assert!(changed(&prepared).contains("Touch(var Value: integer; var Extra: integer)"));
    for argument in ["A", "0", "var B", "B + 1", "Missing"] {
        let refused = prepare(
            &context,
            &recipe("Touch", "Extra", "integer", ParameterMode::Var, argument),
        );
        assert!(refused.plan.is_none(), "{argument}: {refused:?}");
        assert_eq!(std::fs::read_to_string(&path).expect("unchanged"), source);
    }
}

#[test]
fn both_old_body_capture_and_recursive_argument_capture_are_refused() {
    for source in [
        "program Main; const Extra: integer := 7; function Leaf(): integer; begin return Extra; end function; begin discard Leaf(); end.",
        "program Main; const Extra: integer := 7; function Leaf(N: integer): integer; begin if N > 0 then return Leaf(N - 1); end if; return 0; end function; begin discard Leaf(2); end.",
    ] {
        let (_temp, context, path) = fixture(source);
        let refused = prepare(
            &context,
            &recipe("Leaf", "Extra", "integer", ParameterMode::Value, "Extra"),
        );
        assert!(refused.plan.is_none(), "{refused:?}");
        assert!(
            refused.blockers.iter().any(|b| b.code == "binding_changed"),
            "{refused:?}"
        );
        assert_eq!(std::fs::read_to_string(path).expect("unchanged"), source);
    }
}

#[test]
fn callable_values_callbacks_dynamic_spawns_and_generic_owners_block_the_entire_plan() {
    for (source, target, code) in [
        (
            "program Main; function Leaf(): integer; begin return 1; end function; begin const F := Leaf; discard Leaf(); end.",
            "Leaf",
            "callable_value",
        ),
        (
            "program Main; function Leaf(): integer; begin return 1; end function; procedure Accept(F: function(): integer); begin end procedure; begin Accept(Leaf); discard Leaf(); end.",
            "Leaf",
            "callable_value",
        ),
        (
            "program Main; function Leaf(): integer; begin return 1; end function; procedure Invoke(F: function(): integer); begin discard F(); end procedure; begin discard Leaf(); end.",
            "Leaf",
            "dynamic_call",
        ),
        (
            "program Main; function Leaf(): integer; begin return 1; end function; begin scope const Job := go Leaf(); end scope; end.",
            "Leaf",
            "unsupported_invocation",
        ),
        (
            "program Main; function Echo<T>(Value: T): T; begin return Value; end function; begin discard Echo(1); end.",
            "Echo",
            "unsupported_declaration",
        ),
        (
            "program Main; function Leaf(): integer; begin return 1; end function; function Echo<T>(Value: T): integer; begin return Leaf(); end function; begin discard Echo(1); end.",
            "Leaf",
            "generic_call",
        ),
        (
            "program Main; function Leaf(): integer; begin return 1; end function; type Item of T = record Value: T; Seed: integer := Leaf(); end record; begin const A := Item(Value := 1); end.",
            "Leaf",
            "generic_call",
        ),
    ] {
        let (_temp, context, path) = fixture(source);
        let refused = prepare(
            &context,
            &recipe(target, "Extra", "integer", ParameterMode::Value, "0"),
        );
        assert!(refused.plan.is_none(), "{refused:?}");
        assert!(
            refused
                .blockers
                .iter()
                .any(|b| b.code == code && b.start.is_some() && b.path.is_some()),
            "{code}: {refused:?}"
        );
        assert_eq!(std::fs::read_to_string(path).expect("unchanged"), source);
    }
}

#[test]
fn fragments_visibility_duplicate_names_and_self_referential_new_arguments_are_rejected() {
    let source = "program Main; function Leaf(N: integer): integer; begin return N; end function; begin discard Leaf(1); end.";
    let (_temp, context, path) = fixture(source);
    for (name, ty, argument) in [
        ("Extra", "integer; X: integer", "0"),
        ("Extra", "integer", "0, 1"),
        ("Extra", "integer", "0); begin end."),
        ("Extra", "Missing", "0"),
        ("N", "integer", "0"),
        ("begin", "integer", "0"),
        ("Extra", "integer", "Leaf(1, 2)"),
    ] {
        let refused = prepare(
            &context,
            &recipe("Leaf", name, ty, ParameterMode::Value, argument),
        );
        assert!(
            refused.plan.is_none(),
            "{name}/{ty}/{argument}: {refused:?}"
        );
        if argument == "Leaf(1, 2)" {
            assert!(
                refused
                    .blockers
                    .iter()
                    .any(|b| b.code == "recursive_argument" && b.start.is_some()),
                "{refused:?}"
            );
        }
        assert_eq!(std::fs::read_to_string(&path).expect("unchanged"), source);
    }
}

#[test]
fn contracts_bare_cr_nested_parameter_types_and_concrete_generic_types_keep_their_meaning() {
    let source = "program Main;\rfunction Probe(N: integer): integer; begin return N; end function;\rfunction Leaf(First: integer; Second: integer): integer;\rrequires First >= 0;\rensures (Returned): Returned >= 0;\rbegin return First + Second; end function;\rbegin discard Leaf(Second := Probe(2), First := Probe(1)); end.\r";
    let (_temp, context, _) = fixture(source);
    let prepared = prepare(
        &context,
        &recipe("Leaf", "Extra", "integer", ParameterMode::Value, "Probe(3)"),
    );
    let text = changed(&prepared);
    assert!(text.contains("Leaf(Second := Probe(2), First := Probe(1), Extra := Probe(3))"));
    assert!(text.contains("requires First >= 0;\rensures (Returned): Returned >= 0;"));
    assert!(!text.contains('\n'));
    let source = "program Main; type Box of T = record Value: T; end record; function Identity(Value: integer): integer; begin return Value; end function; procedure Leaf(F: function(Value: integer): integer); begin end procedure; begin Leaf(Identity); end.";
    let (_temp, context, _) = fixture(source);
    let prepared = prepare(
        &context,
        &recipe(
            "Leaf",
            "Extra",
            "Box of integer",
            ParameterMode::Value,
            "Box(Value := 1)",
        ),
    );
    assert!(
        changed(&prepared)
            .contains("Leaf(F: function(Value: integer): integer; Extra: Box of integer)")
    );
    assert!(changed(&prepared).contains("Leaf(Identity, Box(Value := 1))"));
}

#[test]
fn implicit_resource_release_and_method_declarations_are_never_treated_as_unused_routines() {
    let source = "program Main; type Handle = record Value: integer; release procedure Close(Self: Handle); begin null; end procedure; end record; begin using const Item := Handle(Value := 1); end.";
    let (_temp, context, _) = fixture(source);
    let refused = prepare(
        &context,
        &recipe(
            "Main.Handle.Close",
            "Extra",
            "integer",
            ParameterMode::Value,
            "0",
        ),
    );
    assert!(refused.plan.is_none(), "{refused:?}");
    assert!(
        refused
            .blockers
            .iter()
            .any(|b| b.code == "unsupported_declaration" && b.start.is_some()),
        "{refused:?}"
    );
}
