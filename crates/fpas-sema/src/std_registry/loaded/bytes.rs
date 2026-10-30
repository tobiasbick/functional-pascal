//! Registration of the native byte-buffer type and its two canonical factories.

use super::super::{define_func, p};
use crate::{
    check::Checker,
    scope::{Symbol, SymbolKind},
    types::Ty,
};

pub(super) fn register(checker: &mut Checker) {
    let error = super::errors::register(
        checker,
        fpas_builtins::native_errors::NativeErrorFamily::Buffer,
    );
    checker.scopes.imports.define_metadata(
        fpas_builtins::std_symbols::STD_BYTES_BYTE_BUFFER,
        Symbol {
            constant: None,
            ty: Ty::ByteBuffer,
            mutable: false,
            kind: SymbolKind::Type,
            capability: crate::TransferCapability::Transferable,
        },
    );
    for (name, parameter) in [
        (
            fpas_builtins::std_symbols::STD_BYTES_CREATE,
            p("Length", Ty::Integer),
        ),
        (
            fpas_builtins::std_symbols::STD_BYTES_FROM_ARRAY,
            p("Values", Ty::Array(Box::new(Ty::Integer))),
        ),
    ] {
        define_func(
            checker,
            name,
            vec![parameter],
            Ty::Result(Box::new(Ty::ByteBuffer), Box::new(error.clone())),
        );
    }
}
