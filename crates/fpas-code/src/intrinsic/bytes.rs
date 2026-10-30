//! Native contiguous byte-buffer intrinsic identifiers.

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// `Std.Bytes` factories and private native receiver implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum BytesIntrinsic {
    Create = 841,
    FromArray = 842,
    Length = 843,
    Capacity = 844,
    ToArray = 845,
    Slice = 846,
    Concat = 847,
    SetAt = 848,
    Fill = 849,
    CopyFrom = 850,
    Resize = 851,
    Reserve = 852,
    TrimCapacity = 853,
    /// Private materialization of already typed HTTP byte chunks.
    JoinChunks = 856,
    /// Private materialization after URI escape and size validation.
    DecodePercent = 857,
}
}
