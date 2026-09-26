//! Shared little-endian encoding primitives and digests used by the compiled unit
//! (`.fpascu`) and program image formats.

mod digest;
mod read;
mod write;

pub use digest::Digest;
pub use read::{ByteReader, ReadError};
pub use write::{write_digest, write_i64, write_u8, write_u16, write_u32, write_u64};
