//! Source-ID assignment for linked project programs.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`, `docs/pascal/program-structure/projects.md`

mod declarations;
mod expressions;
mod statements;
mod support;
mod types;

use fpas_parser::{Program, Unit};

pub(super) fn apply_program_source_id(program: &mut Program, source_id: u32) {
    support::apply_span(&mut program.name_span, source_id);
    for used in &mut program.uses {
        support::apply_qualified_id_source_id(used, source_id);
    }
    for declaration in &mut program.declarations {
        declarations::apply_decl_source_id(declaration, source_id);
    }
    for stmt in &mut program.body {
        statements::apply_stmt_source_id(stmt, source_id);
    }
    support::apply_span(&mut program.span, source_id);
}

pub(super) fn apply_unit_source_id(unit: &mut Unit, source_id: u32) {
    support::apply_qualified_id_source_id(&mut unit.name, source_id);
    for used in &mut unit.uses {
        support::apply_qualified_id_source_id(used, source_id);
    }
    for declaration in &mut unit.declarations {
        declarations::apply_decl_source_id(declaration, source_id);
    }
    support::apply_span(&mut unit.span, source_id);
}
