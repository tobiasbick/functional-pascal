//! Concrete static signatures of callable bindings, fields and indexed elements.
//!
//! **Documentation:** `docs/pascal/language/functions/function-types.md`

use super::native_receiver::receiver_type;
use crate::{CallableSignature, navigation::NavigationDocument};
use fpas_sema::Ty;

/// Describes the callable expression before `(`, using its receiving type.
pub(crate) fn signature(
    documents: &[NavigationDocument],
    target: usize,
    callable: usize,
    parenthesis: usize,
    offset: usize,
) -> Option<CallableSignature> {
    let document = &documents[target];
    let start = crate::navigation::callable_labels::expression_start(document, parenthesis)?;
    let expression = document
        .snapshot
        .source()
        .get(document.tokens[start].span.offset..document.tokens[parenthesis].span.offset)?
        .trim();
    let ty = receiver_type(documents, target, expression, offset, 0)?;
    let display_name = crate::navigation::token_name(document, callable)?;
    let (params, result) = match &ty {
        Ty::Function(function) if !function.variadic => {
            (&function.params, Some(function.return_type.as_ref()))
        }
        Ty::Procedure(procedure) if !procedure.variadic => (&procedure.params, None),
        _ => return None,
    };
    let parameters = params
        .iter()
        .map(|param| {
            format!(
                "{}{}: {}",
                if param.mode == fpas_parser::ParamMode::Var {
                    "var "
                } else {
                    ""
                },
                param.name,
                param.ty
            )
        })
        .collect::<Vec<_>>();
    Some(CallableSignature {
        label: format!(
            "{} {display_name}({}){}",
            if result.is_some() {
                "function"
            } else {
                "procedure"
            },
            parameters.join("; "),
            result.map_or_else(String::new, |ty| format!(": {ty}"))
        ),
        parameters,
    })
}
