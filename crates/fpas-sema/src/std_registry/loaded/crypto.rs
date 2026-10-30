use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_builtins::std_symbols as s;

pub(super) fn register_std_crypto(checker: &mut Checker) {
    let error_ty = super::errors::register(
        checker,
        fpas_builtins::native_errors::NativeErrorFamily::Crypto,
    );
    let error = Box::new(error_ty);
    define_func(
        checker,
        s::STD_CRYPTO_RANDOM_BYTES,
        vec![p("Count", Ty::Integer)],
        Ty::Result(Box::new(Ty::ByteBuffer), error.clone()),
    );
    define_func(
        checker,
        s::STD_CRYPTO_RANDOM_INT,
        vec![p("Lo", Ty::Integer), p("Hi", Ty::Integer)],
        Ty::Result(Box::new(Ty::Integer), error),
    );
}
