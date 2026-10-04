//! Signatures of semantically checked expression-target calls.

use crate::navigation::NavigationDocument;
use crate::{CallableSignature, SignatureHelp};
use fpas_sema::{Ty, ValueCallTarget};

fn target(document: &NavigationDocument, offset: usize) -> Option<&ValueCallTarget> {
    document
        .analysis
        .as_ref()?
        .semantic()?
        .metadata()
        .value_calls
        .values()
        .find(|target| target.call_span.offset == offset)
}

/// Whether the opening parenthesis belongs to a checked callable-value invocation.
pub(super) fn has_value_call(document: &NavigationDocument, offset: usize) -> bool {
    target(document, offset).is_some()
}

/// Build a signature from the callable's explicit positional parameters.
pub(super) fn signature_help(
    document: &NavigationDocument,
    offset: usize,
    argument: usize,
) -> Option<SignatureHelp> {
    let target = target(document, offset)?;
    let (kind, params, result) = match &target.callable_ty {
        Ty::Function(signature) => (
            "function",
            &signature.params,
            format!(": {}", target.result_ty),
        ),
        Ty::Procedure(signature) => ("procedure", &signature.params, String::new()),
        _ => return None,
    };
    let parameters = params
        .iter()
        .map(|param| {
            format!(
                "{}{}: {}",
                if param.mutable { "var " } else { "" },
                param.name,
                param.ty
            )
        })
        .collect::<Vec<_>>();
    let active_parameter = (!parameters.is_empty()).then(|| argument.min(parameters.len() - 1));
    Some(SignatureHelp {
        signature: CallableSignature {
            label: format!("{kind}({}){result}", parameters.join("; ")),
            parameters,
        },
        documentation: None,
        parameter_documentation: vec![None; params.len()],
        active_parameter,
    })
}
