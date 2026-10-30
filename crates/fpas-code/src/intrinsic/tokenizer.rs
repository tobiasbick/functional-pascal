//! Model-tokenization selectors for `Std.Ai.Tokenizer`.

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Deterministic model-tokenization operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum TokenizerIntrinsic {
    /// Count ordinary text tokens for an explicit encoding.
    Count = 837,
    /// Encode ordinary text into model token IDs.
    Encode = 838,
    /// Decode token IDs into validated UTF-8 text.
    Decode = 839,
    /// List the supported encoding names.
    Encodings = 840,
}
}
