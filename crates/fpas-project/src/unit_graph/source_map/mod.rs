//! Source-ID assignment for independently compiled project units.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`, `docs/pascal/program-structure/projects.md`

mod declarations;
mod expressions;
mod statements;
mod support;
mod types;

use fpas_parser::Unit;

pub(crate) fn apply_unit_source_id(unit: &mut Unit, source_id: u32) {
    support::apply_qualified_id_source_id(&mut unit.name, source_id);
    for used in &mut unit.uses {
        support::apply_qualified_id_source_id(used, source_id);
    }
    for declaration in &mut unit.declarations {
        declarations::apply_decl_source_id(declaration, source_id);
    }
    support::apply_span(&mut unit.span, source_id);
}
