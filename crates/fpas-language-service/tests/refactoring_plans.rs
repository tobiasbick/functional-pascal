//! Recipe identity, complete read-set validation and idempotent plan reapplication.

#![allow(
    clippy::expect_used,
    reason = "plan fixtures assert exact prepared states"
)]

use crate::support::{self, TempDirectory};
use fpas_language_service::{
    CancellationToken, RefactorPlan, RefactorPreparation, RefactorRecipe, RefactorStatus,
    SourceSelector, SourceSession, identity_path,
};
use std::path::Path;

fn identity(path: &Path) -> String {
    identity_path(&std::fs::canonicalize(path).expect("physical source")).expect("identity")
}

fn prepare(context: &Path, recipe: &RefactorRecipe) -> RefactorPreparation {
    SourceSession::load(context, None, &[], &CancellationToken::new())
        .expect("scope")
        .prepare_refactor(recipe, &CancellationToken::new())
        .expect("prepare")
}

fn revalidate(
    plan: &RefactorPlan,
) -> Result<RefactorPreparation, fpas_language_service::SourceQueryError> {
    SourceSession::load(
        Path::new(&plan.context.input),
        None,
        &[],
        &CancellationToken::new(),
    )?
    .revalidate_refactor(plan, &CancellationToken::new())
}

fn apply_fixture(prepared: &RefactorPreparation) {
    for change in &prepared.changes.as_ref().expect("checked changes").changes {
        std::fs::write(&change.path, &change.replacement)
            .expect("fixture applies exact checked bytes");
    }
}

#[test]
fn rename_plan_is_deterministic_rechecked_and_recognizes_the_complete_after_state() {
    let temp = TempDirectory::new("rename-plan");
    let (manifest, main, unit) = support::write_program_project(&temp);
    let original = std::fs::read_to_string(&unit).expect("source");
    let recipe = RefactorRecipe::Rename {
        selector: SourceSelector::Symbol {
            name: "Demo.Math.Answer".into(),
        },
        name: "Compute".into(),
    };
    let first = prepare(&manifest, &recipe);
    assert_eq!(first.status, RefactorStatus::Prepared, "{first:?}");
    let plan = first.plan.as_ref().expect("plan");
    assert_eq!(
        plan,
        prepare(&manifest, &recipe)
            .plan
            .as_ref()
            .expect("same plan")
    );
    assert_eq!(
        std::fs::read_to_string(&unit).expect("untouched source"),
        original
    );
    assert!(
        plan.before
            .files
            .iter()
            .any(|f| f.path == identity(&manifest))
    );
    assert_eq!(plan.edits.len(), 2);
    assert_eq!(
        revalidate(plan).expect("before regeneration").plan.as_ref(),
        Some(plan)
    );

    // A partial application cannot authorize finishing a guessed remainder.
    let first_change = &first.changes.as_ref().expect("changes").changes[0];
    std::fs::write(&first_change.path, &first_change.replacement).expect("partial fixture");
    assert_eq!(
        revalidate(plan).expect_err("partial application").code,
        "stale_plan"
    );
    apply_fixture(&first);
    let repeated = revalidate(plan).expect("complete after state");
    assert_eq!(repeated.status, RefactorStatus::AlreadyApplied);
    assert!(repeated.changes.is_none());
    assert!(
        std::fs::read_to_string(main)
            .expect("caller")
            .contains("Math.Compute()")
    );

    // A checksum is not authority: even a recomputed digest cannot legitimize false old bytes.
    let mut corrupt = plan.clone();
    corrupt.edits[0].old_text = "X".repeat(corrupt.edits[0].length);
    corrupt.digest = corrupt.content_digest();
    assert_eq!(
        revalidate(&corrupt).expect_err("corrupt inverse").code,
        "invalid_plan"
    );
}

#[test]
fn unedited_inputs_and_membership_invalidate_both_before_and_after_plans() {
    for after in [false, true] {
        let temp = TempDirectory::new("plan-read-set");
        let (manifest, _, _) = support::write_program_project(&temp);
        let unused = temp.write("src/unused.fpas", "unit Demo.Unused; end unit;");
        let prepared = prepare(
            &manifest,
            &RefactorRecipe::Rename {
                selector: SourceSelector::Symbol {
                    name: "Answer".into(),
                },
                name: "Compute".into(),
            },
        );
        let plan = prepared.plan.as_ref().expect("plan");
        if after {
            apply_fixture(&prepared);
        }
        std::fs::write(&unused, "unit Demo.Unused; // changed\nend unit;")
            .expect("unedited input changes");
        assert_eq!(
            revalidate(plan).expect_err("whole inventory stale").code,
            "stale_plan"
        );
        std::fs::write(&unused, "unit Demo.Unused; end unit;").expect("restore");
        temp.write("src/added.fpas", "unit Demo.Added; end unit;");
        assert_eq!(
            revalidate(plan).expect_err("membership stale").code,
            "stale_plan"
        );
    }
}

#[test]
fn extraction_and_import_removal_are_idempotent_and_preserve_bare_cr() {
    let temp = TempDirectory::new("local-plans");
    let (manifest, main, _) = support::write_program_project(&temp);
    let source =
        "program App;\ruses Demo.Math as M;\rbegin\r  const Output := (1 + 2) * 3;\rend.\r";
    std::fs::write(&main, source).expect("source");
    let start = source.find("(1 + 2)").expect("selection");
    let extracted = prepare(
        &manifest,
        &RefactorRecipe::ExtractVariable {
            path: identity(&main),
            start,
            end: start + 7,
            name: None,
        },
    );
    assert_eq!(extracted.status, RefactorStatus::Prepared, "{extracted:?}");
    let plan = extracted.plan.as_ref().expect("extraction plan");
    assert!(
        matches!(&plan.recipe, RefactorRecipe::ExtractVariable { name: Some(name), .. } if name == "ExtractedValue")
    );
    apply_fixture(&extracted);
    let changed = std::fs::read_to_string(&main).expect("extracted source");
    assert!(!changed.contains('\n'));
    assert!(changed.contains("const ExtractedValue := (1 + 2);\r"));
    assert_eq!(
        revalidate(plan).expect("repeat extraction").status,
        RefactorStatus::AlreadyApplied
    );
    let imports = prepare(
        &manifest,
        &RefactorRecipe::OrganizeImports {
            path: identity(&main),
        },
    );
    assert_eq!(imports.status, RefactorStatus::Prepared, "{imports:?}");
    apply_fixture(&imports);
    assert!(
        !std::fs::read_to_string(&main)
            .expect("organized source")
            .contains("uses ")
    );
    assert_eq!(
        revalidate(imports.plan.as_ref().expect("imports plan"))
            .expect("repeat removal")
            .status,
        RefactorStatus::AlreadyApplied
    );
}

#[test]
fn no_change_unsupported_and_incomplete_are_distinct_checked_outcomes() {
    let temp = TempDirectory::new("plan-outcomes");
    let (manifest, main, _) = support::write_program_project(&temp);
    let recipe = RefactorRecipe::OrganizeImports {
        path: identity(&main),
    };
    let no_change = prepare(&manifest, &recipe);
    assert_eq!(no_change.status, RefactorStatus::NoChange, "{no_change:?}");
    let plan = no_change.plan.as_ref().expect("empty checked plan");
    assert!(plan.edits.is_empty());
    assert_eq!(plan.before, plan.after);
    assert_eq!(
        revalidate(plan).expect("empty plan checks recipe").status,
        RefactorStatus::NoChange
    );

    let mut invalid_recipe = plan.clone();
    invalid_recipe.recipe = RefactorRecipe::Rename {
        selector: SourceSelector::Symbol {
            name: "Missing".into(),
        },
        name: "Compute".into(),
    };
    invalid_recipe.digest = invalid_recipe.content_digest();
    assert_eq!(
        revalidate(&invalid_recipe)
            .expect_err("empty plans still regenerate")
            .code,
        "stale_plan"
    );

    std::fs::write(
        &main,
        "program App; uses Demo.Math as M // keep\n; begin discard M.Answer(); end.",
    )
    .expect("commented imports");
    let unsupported = prepare(&manifest, &recipe);
    assert_eq!(unsupported.status, RefactorStatus::Unsupported);
    assert_eq!(unsupported.blockers[0].code, "import_comments");
    assert!(unsupported.plan.is_none());

    temp.write(
        "src/broken.fpas",
        "unit Demo.Broken; public const Bad := Missing; end unit;",
    );
    let incomplete = prepare(&manifest, &recipe);
    assert_eq!(incomplete.status, RefactorStatus::Incomplete);
    assert!(incomplete.plan.is_none());
    assert!(!incomplete.diagnostics.is_empty());
}

#[test]
fn malformed_or_tampered_edits_never_reach_recipe_regeneration() {
    let temp = TempDirectory::new("invalid-plans");
    let (manifest, _, _) = support::write_program_project(&temp);
    let prepared = prepare(
        &manifest,
        &RefactorRecipe::Rename {
            selector: SourceSelector::Symbol {
                name: "Answer".into(),
            },
            name: "Compute".into(),
        },
    );
    let original = prepared.plan.expect("plan");
    let mut keyword = original.clone();
    if let RefactorRecipe::Rename { name, .. } = &mut keyword.recipe {
        *name = "begin".into();
    }
    keyword.digest = keyword.content_digest();
    let changes = prepared.changes.expect("replacement buffers").changes;
    for change in &changes {
        std::fs::write(&change.path, &change.replacement).expect("expected after state");
    }
    assert_eq!(
        revalidate(&keyword)
            .expect_err("malformed recipes fail even in the expected after state")
            .code,
        "invalid_plan"
    );
    for change in changes {
        std::fs::write(&change.path, change.original.source()).expect("restore before state");
    }
    let mut changed = original.clone();
    changed.edits[0].new_text = "Invalid".into();
    assert_eq!(
        changed.validate().expect_err("digest mismatch").code,
        "invalid_plan"
    );
    let mut overlapping = original.clone();
    overlapping.edits.insert(0, overlapping.edits[0].clone());
    overlapping.digest = overlapping.content_digest();
    assert_eq!(
        overlapping.validate().expect_err("overlap").code,
        "invalid_plan"
    );
    let mut forged = original;
    forged.edits[0].new_text = "ForgedX".into();
    forged.digest = forged.content_digest();
    assert_eq!(
        revalidate(&forged)
            .expect_err("digest cannot authorize arbitrary edits")
            .code,
        "stale_plan"
    );
}
