//! Model-tokenization signatures with explicit encoding selection.

use super::super::{define_func, p};
use crate::{check::Checker, types::Ty};
use fpas_builtins::{native_errors::NativeErrorFamily, std_symbols as s};

pub(super) fn register(checker: &mut Checker) {
    let error = super::errors::register(checker, NativeErrorFamily::Tokenizer);
    let tokens = Ty::Array(Box::new(Ty::Integer));
    for (name, input, output) in [
        (s::STD_TOKENIZER_COUNT, Ty::String, Ty::Integer),
        (s::STD_TOKENIZER_ENCODE, Ty::String, tokens.clone()),
        (s::STD_TOKENIZER_DECODE, tokens, Ty::String),
    ] {
        let argument = if name == s::STD_TOKENIZER_DECODE {
            "Tokens"
        } else {
            "Text"
        };
        define_func(
            checker,
            name,
            vec![p(argument, input), p("Encoding", Ty::String)],
            Ty::Result(Box::new(output), Box::new(error.clone())),
        );
    }
    define_func(
        checker,
        s::STD_TOKENIZER_ENCODINGS,
        vec![],
        Ty::Array(Box::new(Ty::String)),
    );
}
