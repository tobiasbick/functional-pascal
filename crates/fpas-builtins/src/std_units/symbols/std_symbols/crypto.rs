//! `Std.Crypto` symbol names and registry group.

std_symbol!(STD_CRYPTO_RANDOM_BYTES = std_crypto!("RandomBytes"));
std_symbol!(STD_CRYPTO_RANDOM_INT = std_crypto!("RandomInt"));

pub(in crate::std_units) const STD_CRYPTO_SYMBOLS: &[&str] =
    &[STD_CRYPTO_RANDOM_BYTES, STD_CRYPTO_RANDOM_INT];
