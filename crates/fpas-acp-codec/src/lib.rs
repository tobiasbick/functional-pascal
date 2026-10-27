//! SDK-validated ACP payload conversion, independent of FPAS connections and MCP.
pub mod binding;
pub mod codec;
mod normalizers;

pub use fpas_acp_model::{
    CodecError, ErrorKind, RawJson, UInt64, error, protocol, raw, schema, value,
};
