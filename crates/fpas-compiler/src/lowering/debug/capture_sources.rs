//! Exact lexical-owner and capture-source identity for debugger construction.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::collections::HashMap;

use fpas_ir::{
    CaptureKind, DebugBindingId, DebugBindingKind, DebugCaptureSource, Function, FunctionId,
};

use crate::CompileError;
use crate::error::internal_compiler_error;

/// Attach portable capture provenance after every function has been lowered.
///
/// Capture names are resolved to owner binding IDs here. The emitted metadata stores those
/// identities; the debugger never matches captures by display name.
pub(in crate::lowering) fn attach(
    functions: &mut [Function],
    owners: &HashMap<FunctionId, FunctionId>,
) -> Result<(), CompileError> {
    let mut pending = Vec::with_capacity(functions.len());
    for function in functions.iter() {
        pending.push(sources_for(function, functions, owners)?);
    }
    for (function, (owner, sources)) in functions.iter_mut().zip(pending) {
        function.debug.lexical_owner = owner;
        function.debug.capture_sources = sources;
    }
    Ok(())
}

fn sources_for(
    function: &Function,
    functions: &[Function],
    owners: &HashMap<FunctionId, FunctionId>,
) -> Result<(Option<FunctionId>, Vec<DebugCaptureSource>), CompileError> {
    if function.captures.is_empty() {
        return Ok((None, Vec::new()));
    }
    if function.name.starts_with("$bound_") {
        return bound_method_sources(function);
    }
    let owner_id = owners.get(&function.id).copied().ok_or_else(|| {
        internal_compiler_error(
            format!(
                "Capturing function `{}` has no lexical owner metadata.",
                function.name
            ),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            1,
            1,
        )
    })?;
    let owner = functions
        .iter()
        .find(|candidate| candidate.id == owner_id)
        .ok_or_else(|| {
            internal_compiler_error(
                format!(
                    "Lexical owner {} for `{}` is missing from the lowered program.",
                    owner_id.get(),
                    function.name
                ),
                "This is an internal compiler error. Re-run compilation and report the source program.",
                1,
                1,
            )
        })?;
    let capture_bindings = function
        .debug
        .bindings
        .iter()
        .filter(|binding| binding.kind == DebugBindingKind::Capture)
        .collect::<Vec<_>>();
    if capture_bindings.len() != function.captures.len() {
        return Err(internal_compiler_error(
            format!(
                "Function `{}` capture bindings do not match capture declarations.",
                function.name
            ),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            1,
            1,
        ));
    }
    let mut sources = Vec::with_capacity(function.captures.len());
    for (declaration, binding) in function.captures.iter().zip(capture_bindings) {
        let id = resolve_owner_binding(owner, binding, declaration.kind)?;
        sources.push(DebugCaptureSource {
            binding: id,
            ty: declaration.ty,
            kind: declaration.kind,
        });
    }
    Ok((Some(owner_id), sources))
}

fn bound_method_sources(
    function: &Function,
) -> Result<(Option<FunctionId>, Vec<DebugCaptureSource>), CompileError> {
    let capture_bindings = function
        .debug
        .bindings
        .iter()
        .enumerate()
        .filter(|(_, binding)| binding.kind == DebugBindingKind::Capture)
        .collect::<Vec<_>>();
    if capture_bindings.len() != function.captures.len() {
        return Err(internal_compiler_error(
            format!(
                "Bound method `{}` capture bindings do not match capture declarations.",
                function.name
            ),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            1,
            1,
        ));
    }
    let mut sources = Vec::with_capacity(function.captures.len());
    for (declaration, (index, _)) in function.captures.iter().zip(capture_bindings) {
        let binding = DebugBindingId::try_from_index(index).map_err(|error| {
            internal_compiler_error(
                error.to_string(),
                "Split the routine into smaller functions.",
                1,
                1,
            )
        })?;
        sources.push(DebugCaptureSource {
            binding,
            ty: declaration.ty,
            kind: declaration.kind,
        });
    }
    Ok((Some(function.id), sources))
}

fn resolve_owner_binding(
    owner: &Function,
    binding: &fpas_ir::DebugBinding,
    kind: CaptureKind,
) -> Result<DebugBindingId, CompileError> {
    let declaration = binding.declaration.ok_or_else(|| {
        internal_compiler_error(
            format!(
                "Capture `{}` in `{}` has no source declaration identity.",
                binding.name, owner.name
            ),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            1,
            1,
        )
    })?;
    let mut matches = owner
        .debug
        .bindings
        .iter()
        .enumerate()
        .filter(|(_, candidate)| {
            !candidate.hidden
                && candidate.declaration == Some(declaration)
                && candidate.ty == binding.ty
                && match kind {
                    CaptureKind::Value => !candidate.cell_backed,
                    CaptureKind::Cell | CaptureKind::EnclosingCell => candidate.cell_backed,
                }
        });
    let Some((index, _)) = matches.next() else {
        return Err(internal_compiler_error(
            format!(
                "Capture `{}` in `{}` has no owner binding with the same declaration identity.",
                binding.name, owner.name
            ),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            declaration.line(),
            declaration.column(),
        ));
    };
    if matches.next().is_some() {
        return Err(internal_compiler_error(
            format!(
                "Capture `{}` in `{}` matches multiple owner bindings.",
                binding.name, owner.name
            ),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            declaration.line(),
            declaration.column(),
        ));
    }
    DebugBindingId::try_from_index(index).map_err(|error| {
        internal_compiler_error(
            error.to_string(),
            "Split the routine into smaller functions.",
            1,
            1,
        )
    })
}
