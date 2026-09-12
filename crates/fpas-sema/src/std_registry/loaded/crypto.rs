use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

pub(super) fn register_std_crypto(checker: &mut Checker) {
    let error = Box::new(Ty::String);
    define_func(
        checker,
        s::STD_CRYPTO_RANDOM_BYTES,
        vec![p("Count", Ty::Integer, false)],
        Ty::Result(Box::new(Ty::Array(Box::new(Ty::Integer))), error.clone()),
    );
    define_func(
        checker,
        s::STD_CRYPTO_RANDOM_INT,
        vec![p("Lo", Ty::Integer, false), p("Hi", Ty::Integer, false)],
        Ty::Result(Box::new(Ty::Integer), error),
    );
}
