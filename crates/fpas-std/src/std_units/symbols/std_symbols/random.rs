//! `Std.Random` symbol names and registry group.

pub const STD_RANDOM_RANDOM: &str = std_random!("Random");
pub const STD_RANDOM_RANDOM_INT: &str = std_random!("RandomInt");
pub const STD_RANDOM_RANDOMIZE: &str = std_random!("Randomize");

pub(in crate::std_units) const STD_RANDOM_SYMBOLS: &[&str] = &[
    STD_RANDOM_RANDOM,
    STD_RANDOM_RANDOM_INT,
    STD_RANDOM_RANDOMIZE,
];
