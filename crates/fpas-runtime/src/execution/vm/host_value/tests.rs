//! Host boundary regressions using real source callbacks and complete value graphs.

mod callables;
mod deep;
mod resources;
mod roots;
mod selection;
mod shutdown;

use super::*;
use crate::execution::vm::{CallbackSession, RuntimeExecution};
use crate::{FunctionId, VerifiedExecutable};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

fn image(source: &str) -> Arc<VerifiedExecutable> {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    Arc::new(fpas_compiler::compile(&program).expect("host fixture compiles"))
}

fn function(image: &VerifiedExecutable, name: &str) -> FunctionId {
    let index = image
        .executable()
        .functions
        .iter()
        .position(|info| {
            image
                .executable()
                .strings
                .get(info.name)
                .is_some_and(|declared| declared.eq_ignore_ascii_case(name))
        })
        .expect("host fixture function");
    FunctionId::try_from_index(index).expect("function id")
}

fn invoke(
    session: &mut CallbackSession,
    image: &VerifiedExecutable,
    name: &str,
    arguments: Vec<HostArgument>,
) -> HostValue {
    session
        .invoke(function(image, name), arguments)
        .expect("host callback")
        .value
}

fn exported(context: &HostValues, value: Value) -> HostValue {
    context
        .export(RuntimeExecution {
            _roots: None,
            value,
            instruction_count: 0,
        })
        .value
}

fn empty_image() -> Arc<VerifiedExecutable> {
    image("program Empty; begin end.")
}
