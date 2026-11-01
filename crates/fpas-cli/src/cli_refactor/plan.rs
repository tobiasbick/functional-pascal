//! Bounded plan-envelope decoding and adapter position conversion.

use super::super::protocol::*;
use super::{RefactorAction, RefactorCliConfig, failure};
use crate::cli_input::QuerySelector;
use fpas_language_service::{
    RefactorPlan, RefactorRecipe, SourceQueryError, SourceSelector, SourceSession, TargetReference,
    identity_path,
};
use std::path::Path;

pub(super) fn read(path: &Path) -> Result<RefactorPlan, SourceQueryError> {
    let bytes = crate::oneshot::read_bounded(path, super::report::MAX_JSON)
        .map_err(|e| failure("plan_read", e.to_string()))?;
    let report: Report =
        serde_json::from_slice(&bytes).map_err(|e| failure("invalid_plan", e.to_string()))?;
    let Some(Payload::Refactor {
        plan: Some(plan), ..
    }) = report.result
    else {
        return Err(failure(
            "invalid_plan",
            "Expected a complete refactoring preview envelope.",
        ));
    };
    if report.schema_version != 2 || report.outcome != Outcome::Ok || report.exit_code != 0 {
        return Err(failure(
            "invalid_plan",
            "Only a successful schema-2 refactoring envelope can be applied.",
        ));
    }
    plan.validate()?;
    Ok(*plan)
}

pub(super) fn reference(
    config: &RefactorCliConfig,
) -> Result<Option<TargetReference>, SourceQueryError> {
    if let RefactorAction::Rename {
        selector: QuerySelector::Reference(path),
        ..
    }
    | RefactorAction::AddParameter {
        selector: QuerySelector::Reference(path),
        ..
    } = &config.action
    {
        return super::super::worker::read_reference(path).map(Some);
    }
    Ok(None)
}

pub(super) fn source(config: &RefactorCliConfig) -> Option<&Path> {
    match &config.action {
        RefactorAction::Rename {
            selector: QuerySelector::Position { path, .. },
            ..
        }
        | RefactorAction::AddParameter {
            selector: QuerySelector::Position { path, .. },
            ..
        }
        | RefactorAction::ExtractVariable { path, .. }
        | RefactorAction::OrganizeImports { path } => Some(path),
        _ => None,
    }
}

pub(super) fn recipe(
    config: &RefactorCliConfig,
    reference: Option<TargetReference>,
    session: &mut SourceSession,
) -> Result<RefactorRecipe, SourceQueryError> {
    let identity = |path: &Path| {
        std::fs::canonicalize(path)
            .and_then(|p| identity_path(&p))
            .map_err(|e| failure("source_read", e.to_string()))
    };
    match &config.action {
        RefactorAction::Rename { selector, name }
        | RefactorAction::AddParameter { selector, name, .. } => {
            let selector = match selector {
                QuerySelector::Position { path, line, column } => {
                    let snapshot = session.service().snapshot(path)?;
                    let offset = super::super::source_offset(&snapshot, *line, *column)
                        .ok_or_else(|| {
                            failure(
                                "invalid_position",
                                "The source position is outside its line.",
                            )
                        })?;
                    SourceSelector::Position {
                        path: identity(path)?,
                        offset,
                    }
                }
                QuerySelector::Symbol(name) => SourceSelector::Symbol { name: name.clone() },
                QuerySelector::Reference(_) => SourceSelector::Reference {
                    target_ref: Box::new(
                        reference
                            .ok_or_else(|| failure("invalid_request", "Missing reference."))?,
                    ),
                },
                _ => return Err(failure("invalid_request", "Unexpected search selector.")),
            };
            Ok(
                if let RefactorAction::AddParameter {
                    r#type,
                    mode,
                    argument,
                    ..
                } = &config.action
                {
                    RefactorRecipe::AddParameter {
                        selector,
                        name: name.clone(),
                        r#type: r#type.clone(),
                        mode: *mode,
                        argument: argument.clone(),
                    }
                } else {
                    RefactorRecipe::Rename {
                        selector,
                        name: name.clone(),
                    }
                },
            )
        }
        RefactorAction::ExtractVariable { path, start, end } => {
            let snapshot = session.service().snapshot(path)?;
            let offset = |(line, column)| {
                super::super::source_offset(&snapshot, line, column).ok_or_else(|| {
                    failure(
                        "invalid_position",
                        "The extraction range is outside its source.",
                    )
                })
            };
            Ok(RefactorRecipe::ExtractVariable {
                path: identity(path)?,
                start: offset(*start)?,
                end: offset(*end)?,
                name: None,
            })
        }
        RefactorAction::OrganizeImports { path } => Ok(RefactorRecipe::OrganizeImports {
            path: identity(path)?,
        }),
        _ => Err(failure(
            "invalid_request",
            "Apply requires an existing plan.",
        )),
    }
}
