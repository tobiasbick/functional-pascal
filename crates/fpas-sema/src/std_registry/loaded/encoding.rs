//! Bounded binary-text codec signatures and their nominal error family.

use super::super::{define_func, p};
use crate::{check::Checker, types::Ty};
use fpas_std::std_symbols as s;

pub(super) fn register_std_encoding(checker: &mut Checker) {
    let error = super::errors::register(
        checker,
        fpas_std::native_errors::NativeErrorFamily::Encoding,
    );
    let bytes = Ty::Array(Box::new(Ty::Integer));
    for name in [
        s::STD_ENCODING_BASE64_ENCODE,
        s::STD_ENCODING_BASE64_URL_ENCODE,
        s::STD_ENCODING_HEX_ENCODE,
    ] {
        define_func(
            checker,
            name,
            vec![p("Bytes", bytes.clone())],
            Ty::Result(Box::new(Ty::String), Box::new(error.clone())),
        );
    }
    for name in [
        s::STD_ENCODING_BASE64_DECODE,
        s::STD_ENCODING_BASE64_URL_DECODE,
        s::STD_ENCODING_HEX_DECODE,
    ] {
        define_func(
            checker,
            name,
            vec![p("Text", Ty::String)],
            Ty::Result(Box::new(bytes.clone()), Box::new(error.clone())),
        );
    }
}
