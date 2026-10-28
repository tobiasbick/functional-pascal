//! Internal `Std.Http` intrinsic discriminants.

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics used by source-defined `Std.Http` implementation units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum HttpIntrinsic {
    /// Reserve and store one body-stream state.
    ReserveBodyStreamState = 528,
    /// Test whether one body-stream state exists.
    HasBodyStreamState = 529,
    /// Load one body-stream state.
    LoadBodyStreamState = 530,
    /// Replace one body-stream state.
    StoreBodyStreamState = 531,
    /// Reserve and store one SSE-decoder state.
    ReserveSseDecoderState = 532,
    /// Test whether one SSE-decoder state exists.
    HasSseDecoderState = 533,
    /// Load one SSE-decoder state.
    LoadSseDecoderState = 534,
    /// Replace one SSE-decoder state.
    StoreSseDecoderState = 535,
}
}
