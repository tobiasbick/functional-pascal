//! Internal default imports through direct, supporting, and re-exported types.

use super::*;

#[allow(clippy::too_many_arguments)]
/// Import each persisted initializer once, using its implementation signature.
pub(super) fn install(
    ty: &InterfaceType,
    types: &mut TypeTable,
    callables: &mut BTreeMap<String, Callable>,
    plan: &mut ImportPlan,
    stubs: &mut Vec<Function>,
    installed: &mut BTreeSet<String>,
    first_function: u32,
    span: fpas_lexer::Span,
) -> Result<(), CompileError> {
    match ty {
        InterfaceType::Record(record) => {
            for field in &record.fields {
                if let Some(fpas_unit::interface::FieldDefault::Initializer { name, .. }) =
                    &field.default_value
                {
                    let signature = CallableType {
                        pure: false,
                        type_parameters: Vec::new(),
                        parameters: Vec::new(),
                        result: Some(Box::new(field.ty.clone())),
                        variadic: false,
                    };
                    install_callable(
                        name,
                        None,
                        &signature,
                        types,
                        callables,
                        plan,
                        stubs,
                        installed,
                        first_function,
                        span,
                    )?;
                }
            }
        }
        InterfaceType::Array(inner)
        | InterfaceType::Channel(inner)
        | InterfaceType::Option(inner)
        | InterfaceType::Task(inner) => install(
            inner,
            types,
            callables,
            plan,
            stubs,
            installed,
            first_function,
            span,
        )?,
        InterfaceType::Dictionary(left, right) | InterfaceType::Result(left, right) => {
            install(
                left,
                types,
                callables,
                plan,
                stubs,
                installed,
                first_function,
                span,
            )?;
            install(
                right,
                types,
                callables,
                plan,
                stubs,
                installed,
                first_function,
                span,
            )?;
        }
        _ => {}
    }
    Ok(())
}
