//! Signature labels, active parameters, and precise parameter label ranges.

use fpas_language_service::SignatureHelp as ServiceSignatureHelp;
use tower_lsp_server::ls_types::{
    ParameterInformation, ParameterLabel, SignatureHelp, SignatureInformation,
};

pub(crate) fn signature_help(value: ServiceSignatureHelp) -> SignatureHelp {
    let parameters = value
        .signature
        .parameters
        .iter()
        .map(|parameter| ParameterInformation {
            label: parameter_offsets(&value.signature.label, parameter).map_or_else(
                || ParameterLabel::Simple(parameter.clone()),
                ParameterLabel::LabelOffsets,
            ),
            documentation: None,
        })
        .collect();
    SignatureHelp {
        signatures: vec![SignatureInformation {
            label: value.signature.label,
            documentation: None,
            parameters: Some(parameters),
            active_parameter: value
                .active_parameter
                .and_then(|value| u32::try_from(value).ok()),
        }],
        active_signature: Some(0),
        active_parameter: value
            .active_parameter
            .and_then(|value| u32::try_from(value).ok()),
    }
}

fn parameter_offsets(label: &str, parameter: &str) -> Option<[u32; 2]> {
    let start = label.find(parameter)?;
    let end = start.saturating_add(parameter.len());
    Some([
        u32::try_from(label[..start].encode_utf16().count()).ok()?,
        u32::try_from(label[..end].encode_utf16().count()).ok()?,
    ])
}
