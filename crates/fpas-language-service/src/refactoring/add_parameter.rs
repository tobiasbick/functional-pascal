//! Append-only signature changes with complete use eligibility and capture checks.

mod edits;
mod facts;
pub(super) mod fragments;

use crate::{
    CallRelation, CallRelationKind, CallableKind, CancellationToken, CheckedChanges,
    InvocationKind, RefactorBlocker, RefactorRecipe, SourceQueryError, SourceSelector,
    SourceSession, SymbolKind,
};
use edits::Insertions;
use facts::Facts;
use fragments::Parameter;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) fn prepare(
    session: &mut SourceSession,
    recipe: &RefactorRecipe,
    blockers: &mut Vec<RefactorBlocker>,
    cancellation: &CancellationToken,
) -> Result<(RefactorRecipe, CheckedChanges), SourceQueryError> {
    let RefactorRecipe::AddParameter {
        selector,
        name,
        r#type,
        mode,
        argument,
    } = recipe
    else {
        return Err(SourceQueryError::new(
            "invalid_request",
            "Expected a parameter recipe.",
        ));
    };
    let parameter = Parameter {
        name,
        type_text: r#type,
        mode: (*mode).into(),
        argument,
    };
    parameter.validate()?;
    let target = session.refactor_target(selector, cancellation)?;
    let reference = target.target_ref.ok_or_else(|| {
        SourceQueryError::new(
            "unsupported_target",
            "A source declaration with complete checked identity is required.",
        )
    })?;
    let path = session.editable_source(Path::new(&reference.file))?;
    let symbol = target.symbol.ok_or_else(|| {
        SourceQueryError::new(
            "unsupported_target",
            "The selected declaration has no source symbol.",
        )
    })?;
    let snapshot = target.snapshot.ok_or_else(|| {
        SourceQueryError::new(
            "unsupported_target",
            "The selected declaration has no source snapshot.",
        )
    })?;
    let callable = session.callable_at(&path, symbol.selection_span.offset(), cancellation)?;
    if !matches!(symbol.kind, SymbolKind::Function | SymbolKind::Procedure)
        || callable
            .as_ref()
            .is_none_or(|c| c.generic || c.identity.kind != CallableKind::Routine)
    {
        blockers.push(site(
            "unsupported_declaration",
            "Only named non-generic functions and procedures outside generic owners are supported.",
            &path,
            Some(symbol.selection_span.offset()),
            Some(symbol.selection_span.length()),
        )?);
        return Err(SourceQueryError::new(
            "unsupported_declaration",
            "The declaration is outside the supported signature-change forms.",
        ));
    }
    let mut calls = BTreeMap::<_, CallRelation>::new();
    let coverage = session.visit_call_relations(None, cancellation, |relation| {
        let selected = relation.target.as_ref().is_some_and(|t| {
            t.identity.file.as_deref() == Some(reference.file.as_str())
                && t.identity.start == Some(symbol.full_span.offset())
        });
        let reason = if relation.kind == CallRelationKind::DynamicCall {
            Some((
                "dynamic_call",
                "An unresolved invocation may refer to the changed routine.",
            ))
        } else if !selected {
            None
        } else if relation.kind != CallRelationKind::DirectCall {
            Some((
                "callable_value",
                "A callable-value or callback use prevents complete signature propagation.",
            ))
        } else if relation.invocation != InvocationKind::Ordinary || relation.implicit_receiver {
            Some((
                "unsupported_invocation",
                "Spawn and receiver-based invocations are unsupported.",
            ))
        } else if relation.containing.generic {
            Some((
                "generic_call",
                "An affected call belongs to a generic body or default.",
            ))
        } else if !relation.arguments_complete {
            Some((
                "argument_mapping_unavailable",
                "The written arguments have no complete checked parameter mapping.",
            ))
        } else {
            None
        };
        if let Some((code, message)) = reason {
            blockers.push(site(
                code,
                message,
                relation.snapshot.path(),
                Some(relation.span.offset()),
                Some(relation.span.length()),
            )?);
            if blockers.len() > 8192 {
                return Err(SourceQueryError::new(
                    "plan_limit",
                    "Too many blocking sites for one parameter plan.",
                ));
            }
        } else if selected {
            let key = (
                relation.snapshot.path().to_path_buf(),
                relation.span.offset(),
                relation.span.length(),
            );
            calls.entry(key).or_insert(relation);
            if calls.len() > 65536 {
                return Err(SourceQueryError::new(
                    "plan_limit",
                    "Too many affected call sites for one parameter plan.",
                ));
            }
        }
        Ok(())
    })?;
    if !coverage.reasons.is_empty() {
        return Err(SourceQueryError {
            code: "analysis_incomplete",
            message: "Parameter insertion requires the complete checked call inventory.".into(),
            diagnostics: coverage.diagnostics,
        });
    }
    if !blockers.is_empty() {
        return Err(SourceQueryError::new(
            "unsupported_uses",
            "Some declaration uses cannot be propagated safely.",
        ));
    }
    let calls = calls.into_values().collect::<Vec<_>>();
    for call in &calls {
        session.editable_source(call.snapshot.path())?;
    }
    let scope = session.context().clone();
    let before = Facts::collect(session.service(), &scope, None, cancellation)?;
    let fresh = fresh_name(session, r#type, argument, cancellation)?;
    let probe_parameter = Parameter {
        name: &fresh,
        ..parameter
    };
    let probe = Insertions::generate(&symbol, &snapshot, &calls, &probe_parameter)?;
    let (_, mut probe_service) =
        session
            .service()
            .checked_edit_overlay(&path, &probe.edits(), cancellation)?;
    let probe_facts = Facts::collect(&mut probe_service, &scope, Some(&probe), cancellation)?;
    probe_facts.reject_self_argument(
        &reference.file,
        symbol.full_span.offset(),
        &symbol.qualified_name,
        blockers,
    )?;
    drop(probe_service);
    let final_edits = Insertions::generate(&symbol, &snapshot, &calls, &parameter)?;
    let (changes, mut after_service) =
        session
            .service()
            .checked_edit_overlay(&path, &final_edits.edits(), cancellation)?;
    let after = Facts::collect(&mut after_service, &scope, Some(&final_edits), cancellation)?;
    before.compare_original(&after, blockers)?;
    probe_facts.compare_arguments(&after, blockers)?;
    Ok((
        RefactorRecipe::AddParameter {
            selector: SourceSelector::Reference {
                target_ref: Box::new(reference),
            },
            name: name.clone(),
            r#type: r#type.clone(),
            mode: *mode,
            argument: argument.clone(),
        },
        changes,
    ))
}

fn fresh_name(
    session: &mut SourceSession,
    type_text: &str,
    argument: &str,
    cancellation: &CancellationToken,
) -> Result<String, SourceQueryError> {
    let paths = session
        .snapshot()
        .inventory
        .files
        .iter()
        .filter(|f| {
            Path::new(&f.path)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("fpas"))
        })
        .map(|f| f.path.clone())
        .collect::<Vec<_>>();
    // One scan avoids rescanning a whole context for every occupied generated name.
    let mut occupied = BTreeSet::new();
    let mut scan = |source: &str| {
        for token in fpas_lexer::lex(source).0 {
            if let fpas_lexer::Token::Ident(name) = token.token
                && let Some(suffix) = name.to_ascii_lowercase().strip_prefix("fpasaddedparameter")
                && let Ok(number) = suffix.parse::<usize>()
            {
                occupied.insert(number);
            }
        }
    };
    scan(type_text);
    scan(argument);
    for path in &paths {
        cancellation.check()?;
        scan(session.service().snapshot(Path::new(path))?.source());
    }
    let suffix = (0..=occupied.len())
        .find(|number| !occupied.contains(number))
        .ok_or_else(|| {
            SourceQueryError::new(
                "plan_limit",
                "No fresh parameter name fits the input budget.",
            )
        })?;
    Ok(format!("FpasAddedParameter{suffix}"))
}

fn site(
    code: &str,
    message: &str,
    path: &Path,
    start: Option<usize>,
    length: Option<usize>,
) -> Result<RefactorBlocker, SourceQueryError> {
    Ok(RefactorBlocker {
        code: code.into(),
        message: message.into(),
        path: Some(
            crate::identity_path(path)
                .map_err(|e| SourceQueryError::new("unsupported_path_identity", e.to_string()))?,
        ),
        start,
        length,
    })
}
