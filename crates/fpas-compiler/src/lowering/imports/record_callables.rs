//! Imported record callable registration.
//! See `docs/pascal/program-structure/units.md`.

use super::super::context::Callable;
use super::super::types::TypeTable;
use super::{ImportPlan, install_callable};
use crate::CompileError;
use fpas_ir::Function;
use fpas_unit::interface::RecordType;
use std::collections::{BTreeMap, BTreeSet};

#[allow(clippy::too_many_arguments)]
/// Installs public record methods and static routines from dependency interfaces.
/// See `docs/pascal/program-structure/units.md`.
pub(super) fn install_record_callables(
    record: &RecordType,
    types: &mut TypeTable,
    callables: &mut BTreeMap<String, Callable>,
    plan: &mut ImportPlan,
    stubs: &mut Vec<Function>,
    installed: &mut BTreeSet<String>,
    first_function: u32,
    span: fpas_lexer::Span,
) -> Result<(), CompileError> {
    for method in record.methods.iter().chain(&record.static_routines) {
        if record
            .private_members
            .iter()
            .any(|private| private.eq_ignore_ascii_case(&method.name))
        {
            continue;
        }
        install_callable(
            &format!("{}.{}", record.name, method.name),
            None,
            &method.callable,
            types,
            callables,
            plan,
            stubs,
            installed,
            first_function,
            span,
        )?;
    }
    Ok(())
}
