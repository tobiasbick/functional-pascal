//! `Std.Random` symbol names and registry group.

std_symbol!(STD_RANDOM_RANDOM = std_random!("Random"));
std_symbol!(STD_RANDOM_RANDOM_INT = std_random!("RandomInt"));
std_symbol!(STD_RANDOM_RANDOMIZE = std_random!("Randomize"));

pub(in crate::std_units) const STD_RANDOM_SYMBOLS: &[&str] = &[
    STD_RANDOM_RANDOM,
    STD_RANDOM_RANDOM_INT,
    STD_RANDOM_RANDOMIZE,
];
