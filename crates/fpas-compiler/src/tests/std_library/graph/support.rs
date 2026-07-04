//! Shared helpers for `Std.Graph` compiler integration tests.

use fpas_std::{GraphEvent, with_headless_graph_backend_for_tests};

use super::super::super::compile_ok;

pub(super) const GRAPH_BASICS_EXAMPLE: &str =
    include_str!("../../../../../../examples/pascal/std/graph_basics.fpas");
pub(super) const JULIA_GRAPH_EXAMPLE: &str =
    include_str!("../../../../../../examples/math/julia/julia_graph.fpas");
pub(super) const MANDELBROT_GRAPH_EXAMPLE: &str =
    include_str!("../../../../../../examples/math/mandelbrot/mandelbrot_graph.fpas");

pub(super) fn with_headless<T>(f: impl FnOnce() -> T) -> T {
    with_headless_graph_backend_for_tests(f)
}

pub(super) fn compile_run_with_graph_events(
    source: &str,
    events: &[GraphEvent],
) -> fpas_vm::VmOutput {
    let chunk = compile_ok(source);
    let mut vm = fpas_vm::Vm::new(chunk);
    for event in events {
        vm.push_graph_event(event.clone());
    }
    vm.run().expect("VM should not error");
    vm.output().clone()
}
