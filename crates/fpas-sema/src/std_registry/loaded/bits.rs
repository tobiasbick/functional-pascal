//! `Std.Bits` signatures; see `docs/pascal/std/numeric/bits.md`.
use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Registers side-effect-free bit operations with explicit integer arguments.
pub(super) fn register(checker: &mut Checker) {
    define_func(
        checker,
        s::STD_BITS_BIT_NOT,
        vec![p("Value", Ty::Integer, false)],
        Ty::Integer,
    );
    for name in [s::STD_BITS_BIT_AND, s::STD_BITS_BIT_OR, s::STD_BITS_BIT_XOR] {
        define_func(
            checker,
            name,
            vec![
                p("Left", Ty::Integer, false),
                p("Right", Ty::Integer, false),
            ],
            Ty::Integer,
        );
    }
    for name in [s::STD_BITS_SHIFT_LEFT, s::STD_BITS_SHIFT_RIGHT] {
        define_func(
            checker,
            name,
            vec![
                p("Value", Ty::Integer, false),
                p("Count", Ty::Integer, false),
            ],
            Ty::Integer,
        );
    }
}
