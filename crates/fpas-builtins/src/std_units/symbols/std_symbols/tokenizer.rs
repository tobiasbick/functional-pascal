//! Canonical model-tokenizer symbols.

std_symbol!(STD_TOKENIZER_COUNT = "Std.Ai.Tokenizer.Count");
std_symbol!(STD_TOKENIZER_ENCODE = "Std.Ai.Tokenizer.Encode");
std_symbol!(STD_TOKENIZER_DECODE = "Std.Ai.Tokenizer.Decode");
std_symbol!(STD_TOKENIZER_ENCODINGS = "Std.Ai.Tokenizer.Encodings");

pub(in crate::std_units) const STD_TOKENIZER_SYMBOLS: &[&str] = &[
    STD_TOKENIZER_COUNT,
    STD_TOKENIZER_ENCODE,
    STD_TOKENIZER_DECODE,
    STD_TOKENIZER_ENCODINGS,
];
