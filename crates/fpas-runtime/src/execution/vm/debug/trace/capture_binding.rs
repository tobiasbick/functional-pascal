//! Exact capture ABI/provenance validation, without looking up live owner frames.

use super::model::DebugTraceCaptureSource;
use crate::{DebugBindingKind, DebugCaptureKind, Executable};
use std::collections::BTreeSet;

pub(super) struct Capture<'a> {
    pub type_name: &'a str,
    pub source: DebugTraceCaptureSource,
    pub depth: usize,
}

pub(super) fn resolve(
    image: &Executable,
    mut function: usize,
    mut binding: usize,
) -> Result<Option<Capture<'_>>, String> {
    let mut seen = BTreeSet::new();
    let mut selected_slot = None;
    loop {
        if !seen.insert((function, binding)) {
            return Err("Trace capture provenance is cyclic.".into());
        }
        let info = image
            .functions
            .get(function)
            .ok_or("Missing capture owner")?;
        let local = info
            .debug
            .bindings
            .get(binding)
            .ok_or("Missing capture binding")?;
        if local.hidden {
            return Err("Trace capture provenance reaches a hidden binding.".into());
        }
        if local.kind != DebugBindingKind::Capture {
            return Ok(Some(Capture {
                depth: seen.len() - 1,
                type_name: image
                    .strings
                    .get(local.type_name)
                    .ok_or("Missing capture source type name")?,
                source: DebugTraceCaptureSource {
                    function_id: u16::try_from(function)
                        .map_err(|_| "Capture owner exceeds function ID range")?,
                    binding_id: binding,
                    slot: selected_slot.ok_or("Missing capture ABI slot")?,
                },
            }));
        }
        let captures: Vec<_> = info
            .debug
            .bindings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.kind == DebugBindingKind::Capture)
            .collect();
        if captures.len() != usize::from(info.capture_count)
            || info.debug.capture_sources.len() != captures.len()
        {
            return Err("Trace capture requires complete ABI and provenance metadata.".into());
        }
        let slot = captures
            .iter()
            .position(|(id, _)| *id == binding)
            .ok_or("Missing capture ABI mapping")?;
        selected_slot.get_or_insert(slot);
        let source = &info.debug.capture_sources[slot];
        let owner = usize::from(
            info.debug
                .lexical_owner
                .ok_or("Missing capture lexical owner")?
                .get(),
        );
        let owner_info = image
            .functions
            .get(owner)
            .ok_or("Missing capture lexical owner")?;
        let source_id = source.binding.get() as usize;
        let original = owner_info
            .debug
            .bindings
            .get(source_id)
            .ok_or("Missing capture source binding")?;
        let representation = match source.kind {
            DebugCaptureKind::Value => !local.cell_backed && !original.cell_backed,
            DebugCaptureKind::Cell => {
                local.cell_backed
                    && original.cell_backed
                    && original.kind != DebugBindingKind::Capture
            }
            DebugCaptureKind::EnclosingCell => {
                local.cell_backed
                    && original.cell_backed
                    && original.kind == DebugBindingKind::Capture
            }
        };
        if source.ty != local.ty
            || source.ty != original.ty
            || local.projection != original.projection
            || !representation
            || local.declaration.is_none()
            || local.declaration != original.declaration
            || !image
                .strings
                .get(local.name)
                .zip(image.strings.get(original.name))
                .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b))
        {
            return Err(
                "Trace capture binding does not match its exact source type/representation.".into(),
            );
        }
        if original.kind == DebugBindingKind::Capture
            && super::capture_visibility::shadowed(image, function, owner_info, local.name)?
        {
            return Ok(None);
        }
        function = owner;
        binding = source_id;
    }
}
