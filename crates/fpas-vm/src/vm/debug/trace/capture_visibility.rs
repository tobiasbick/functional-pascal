//! Source visibility at closure definition, independent of later invocation sites.

use crate::vm::inspection::location;
use fpas_bytecode::{DebugBindingKind, Executable, FunctionInfo, Opcode};

pub(super) fn shadowed(
    image: &Executable,
    child_id: usize,
    owner: &FunctionInfo,
    name: fpas_bytecode::StringId,
) -> Result<bool, String> {
    let direct: Vec<_> = owner
        .debug
        .bindings
        .iter()
        .filter(|b| {
            !b.hidden
                && b.kind != DebugBindingKind::Capture
                && image
                    .strings
                    .get(b.name)
                    .zip(image.strings.get(name))
                    .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b))
        })
        .collect();
    if direct.iter().any(|b| b.kind == DebugBindingKind::Parameter) {
        return Ok(true);
    }
    if direct.is_empty() {
        return Ok(false);
    }
    let child = &image.functions[child_id];
    let site = child
        .debug
        .sequence_points
        .first()
        .ok_or("Capture shadowing requires definition source/scope metadata")?
        .location;
    // An inline callback is created in its owner before its own source body.
    // Named nested routines are declared before the owner body; their later
    // constructions/calls do not change the declaration's root lexical scope.
    let mut creation = None;
    for address in owner.code.start.get()..owner.code.end.get() {
        let word = image.code[address as usize];
        if word.opcode() != Ok(Opcode::MakeClosure) || usize::from(word.abc_payload().b) != child_id
        {
            continue;
        }
        let points = &owner.debug.sequence_points;
        let count = points.partition_point(|p| p.instruction.get() <= address);
        let Some(point) = count.checked_sub(1).and_then(|n| points.get(n)) else {
            return Err("Capture shadowing requires closure creation source/scope metadata".into());
        };
        if point.location.source == site.source
            && (point.location.line, point.location.column) <= (site.line, site.column)
        {
            if creation.is_some_and(|previous| previous != (point.scope, point.location)) {
                return Err("Capture definition scope is ambiguous".into());
            }
            creation = Some((point.scope, point.location));
        }
    }
    if let Some((scope, position)) = creation {
        let visible = location::visible_scope_chain(owner, scope);
        for binding in direct {
            if !visible.contains(&binding.scope) {
                continue;
            }
            let declaration = binding
                .declaration
                .ok_or("Capture shadowing requires exact local declaration metadata")?;
            if declaration.source != position.source {
                return Err("Capture shadowing source metadata is inconsistent".into());
            }
            if (declaration.line, declaration.column) <= (position.line, position.column) {
                return Ok(true);
            }
        }
        Ok(false)
    } else {
        Ok(direct.iter().any(|b| b.scope == 0))
    }
}
