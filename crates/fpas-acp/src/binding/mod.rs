//! Shared protocol declarations and detached values for language adapters.

mod build;
mod decode;
mod encode;
mod model;
mod names;
mod roots;
mod value;
mod wire;
pub use model::*;
pub use roots::DecodedPayload;
pub use value::Value;

use crate::{CodecError, schema::Version};
use std::sync::LazyLock;

/// The complete declaration graph derived from the pinned SDK.
pub fn catalog(version: Version) -> Result<&'static Catalog, CodecError> {
    static V1: LazyLock<Result<Catalog, CodecError>> =
        LazyLock::new(|| build::catalog(Version::V1));
    static V2: LazyLock<Result<Catalog, CodecError>> =
        LazyLock::new(|| build::catalog(Version::V2));
    match version {
        Version::V1 => V1.as_ref(),
        Version::V2 => V2.as_ref(),
    }
    .map_err(Clone::clone)
}
