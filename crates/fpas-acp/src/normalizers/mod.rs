//! Direct SDK serde validators. Regenerate the tables when changing the protocol pin.

use crate::{CodecError, RawJson, codec, schema::Version};
use serde::{Serialize, de::DeserializeOwned};

type Normalize = fn(&RawJson) -> Result<RawJson, CodecError>;

fn typed<T: Serialize + DeserializeOwned>(raw: &RawJson) -> Result<RawJson, CodecError> {
    let value: T = codec::deserialize(raw)?;
    codec::encode(&value, crate::raw::MAX_RAW_BYTES)
}

macro_rules! normalizers {
    ($version:ident; $($name:ident),* $(,)?) => {
        pub(super) fn lookup(name: &str) -> Option<super::Normalize> {
            match name {
                $(stringify!($name) => Some(super::typed::<crate::protocol::$version::$name>),)*
                _ => None,
            }
        }
    };
}

mod v1;
mod v2;

pub(crate) fn lookup(version: Version, name: &str) -> Option<Normalize> {
    if name == "ProtocolVersion" {
        return Some(typed::<crate::protocol::ProtocolVersion>);
    }
    match version {
        Version::V1 => v1::lookup(name),
        Version::V2 => v2::lookup(name),
    }
}
