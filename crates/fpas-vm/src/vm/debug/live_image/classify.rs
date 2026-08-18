//! Compare a candidate executable with the live image without replacing it.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::collections::{BTreeMap, BTreeSet};

use fpas_bytecode::{Executable, FunctionId, VerifiedExecutable};

use super::class::{LiveImageClassification, LiveImageUpdateClass};
use super::fingerprint::{
    capture_identity, debug_identity, entry_name, enum_layouts, function_body_identity,
    function_names, global_layouts, named_functions, record_layouts, signature_identity,
    source_map_identity,
};

/// Classify `candidate` against `current` using live stack function identities.
///
/// The comparison does not mutate either image. Missing rules stay named
/// rejects rather than a live-image claim.
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[must_use]
pub fn classify_live_image(
    current: &VerifiedExecutable,
    candidate: &VerifiedExecutable,
    active: &BTreeSet<FunctionId>,
) -> LiveImageClassification {
    classify(current.executable(), candidate.executable(), active)
}

fn classify(
    current: &Executable,
    candidate: &Executable,
    active: &BTreeSet<FunctionId>,
) -> LiveImageClassification {
    if entry_name(current) != entry_name(candidate) {
        return LiveImageClassification::new(LiveImageUpdateClass::EntryPoint);
    }

    let current_names = function_names(current);
    let candidate_names = function_names(candidate);
    if current_names != candidate_names {
        if added_capturing_function(current, candidate) {
            return LiveImageClassification::new(LiveImageUpdateClass::AnonymousClosure);
        }
        return LiveImageClassification::new(LiveImageUpdateClass::FunctionSet);
    }

    if record_layouts(current) != record_layouts(candidate) {
        return LiveImageClassification::new(LiveImageUpdateClass::RecordLayout);
    }
    if enum_layouts(current) != enum_layouts(candidate) {
        return LiveImageClassification::new(LiveImageUpdateClass::EnumLayout);
    }
    if global_layouts(current) != global_layouts(candidate) {
        return LiveImageClassification::new(LiveImageUpdateClass::GlobalLayout);
    }

    let Some(current_functions) = unique_functions(current) else {
        return LiveImageClassification::new(LiveImageUpdateClass::FunctionSet);
    };
    let Some(candidate_functions) = unique_functions(candidate) else {
        return LiveImageClassification::new(LiveImageUpdateClass::FunctionSet);
    };

    let mut body_changed = BTreeSet::new();
    let mut debug_changed = false;
    for (name, (id, current_function)) in &current_functions {
        let Some((_, candidate_function)) = candidate_functions.get(name) else {
            return LiveImageClassification::new(LiveImageUpdateClass::FunctionSet);
        };
        if capture_identity(current, current_function)
            != capture_identity(candidate, candidate_function)
        {
            return LiveImageClassification::new(LiveImageUpdateClass::ClosureCapture);
        }
        if current_function.flags.uses_spawn_tasks != candidate_function.flags.uses_spawn_tasks {
            return LiveImageClassification::new(LiveImageUpdateClass::TaskIdentity);
        }
        if signature_identity(current_function) != signature_identity(candidate_function) {
            return LiveImageClassification::new(LiveImageUpdateClass::FunctionSet);
        }
        let current_body = function_body_identity(current, current_function);
        let candidate_body = function_body_identity(candidate, candidate_function);
        let body_differs = current_body != candidate_body
            || current_function.register_count != candidate_function.register_count;
        if body_differs {
            body_changed.insert(*id);
        } else if debug_identity(current, current_function)
            != debug_identity(candidate, candidate_function)
        {
            debug_changed = true;
        }
    }

    if body_changed.iter().any(|id| active.contains(id)) {
        return LiveImageClassification::new(LiveImageUpdateClass::ActiveFunctionBody);
    }
    if debug_changed
        || source_map_identity(current, &body_changed)
            != source_map_identity(candidate, &body_changed)
    {
        return LiveImageClassification::new(LiveImageUpdateClass::DebugMetadata);
    }
    if !body_changed.is_empty() {
        return LiveImageClassification::new(LiveImageUpdateClass::InactiveFunctionBody);
    }
    LiveImageClassification::new(LiveImageUpdateClass::Unchanged)
}

fn unique_functions(
    image: &Executable,
) -> Option<BTreeMap<String, (FunctionId, &fpas_bytecode::FunctionInfo)>> {
    let mut functions = BTreeMap::new();
    for (name, id, function) in named_functions(image) {
        if functions.insert(name, (id, function)).is_some() {
            return None;
        }
    }
    Some(functions)
}

fn added_capturing_function(current: &Executable, candidate: &Executable) -> bool {
    let current_names: BTreeSet<String> = function_names(current).into_iter().collect();
    named_functions(candidate)
        .into_iter()
        .any(|(name, _, function)| !current_names.contains(&name) && function.capture_count > 0)
}
