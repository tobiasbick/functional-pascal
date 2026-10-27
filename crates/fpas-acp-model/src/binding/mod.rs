//! SDK-derived declaration catalog shared by the compiler and protocol codecs.
mod build;
mod model;
mod names;
pub use model::*;

use crate::{CodecError, schema::Version};
use std::sync::LazyLock;

/// The complete declaration graph derived from the pinned SDK schema.
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
