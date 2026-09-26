//! IR optimization passes that run between lowering and bytecode selection.
//!
//! Passes keep every instruction's block and index, or remap debugger locations they move, so
//! sequence points and binding initializers stay valid.

mod constant_folding;
mod loop_constants;

use fpas_ir::Program;

/// Run all IR optimization passes on `program` in place.
pub(crate) fn optimize(program: &mut Program) {
    for function in &mut program.functions {
        constant_folding::fold_constants(function);
    }
    loop_constants::hoist_loop_constants(program);
}
