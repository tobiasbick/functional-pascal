//! Conversion of protocol-independent IntelliSense results to LSP values.

mod completion;
mod signature_help;

pub(crate) use completion::CompletionResolveIdentity;

pub(crate) use completion::{completion_item, resolve_completion_item, resolve_identity};
pub(crate) use signature_help::signature_help;
