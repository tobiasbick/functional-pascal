//! Source spelling of generic native type declarations.

use fpas_sema::GenericParamDef;

pub(super) fn declaration(name: &str, parameters: &[GenericParamDef]) -> String {
    let parameters = parameters
        .iter()
        .map(|parameter| {
            let name = parameter.display_name();
            parameter.constraint.map_or_else(
                || name.to_owned(),
                |constraint| format!("{name}: {}", constraint.display_name()),
            )
        })
        .collect::<Vec<_>>();
    match parameters.as_slice() {
        [] => name.to_owned(),
        [parameter] => format!("{name} of {parameter}"),
        _ => format!("{name} of ({})", parameters.join(", ")),
    }
}
