//! Canonical public `Std.Bytes` factories and type identity.

std_symbol!(STD_BYTES_BYTE_BUFFER = "Std.Bytes.ByteBuffer");
std_symbol!(STD_BYTES_CREATE = "Std.Bytes.Create");
std_symbol!(STD_BYTES_FROM_ARRAY = "Std.Bytes.FromArray");

pub(in crate::std_units) const STD_BYTES_SYMBOLS: &[&str] = &[
    STD_BYTES_BYTE_BUFFER,
    STD_BYTES_CREATE,
    STD_BYTES_FROM_ARRAY,
];
