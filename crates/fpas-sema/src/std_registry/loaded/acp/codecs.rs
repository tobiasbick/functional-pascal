//! Public, typed payload codec signatures; these operations publish no messages.

use super::super::super::{define_func, p};
use crate::{check::Checker, types::Ty};
use fpas_acp::{
    binding::{self, ROOTS},
    schema::Version,
};

pub(super) fn register(checker: &mut Checker) {
    let raw = checker
        .scopes
        .imports
        .lookup_metadata("Std.Acp.Types.RawJson")
        .expect("RawJson")
        .ty
        .clone();
    let error = checker
        .scopes
        .imports
        .lookup_metadata("Std.Acp.Types.AcpError")
        .expect("AcpError")
        .ty
        .clone();
    for version in [Version::V1, Version::V2] {
        let catalog = binding::catalog(version).expect("pinned SDK catalog");
        for root in ROOTS {
            let name = &catalog.declarations[root].name;
            let Some(symbol) = checker.scopes.imports.lookup_metadata(name) else {
                continue;
            };
            let payload = symbol.ty.clone();
            let module = if version == Version::V1 {
                "Std.Acp.Types"
            } else {
                "Std.Acp.V2"
            };
            let decoded = super::super::type_registration::register_record_type(
                checker,
                &format!("{module}.Decoded{root}"),
                vec![
                    ("Payload".into(), payload.clone()),
                    ("RawPayload".into(), raw.clone()),
                ],
            );
            define_func(
                checker,
                &format!("{module}.Decode{root}"),
                vec![p("Method", Ty::String), p("Payload", raw.clone())],
                Ty::Result(Box::new(decoded), Box::new(error.clone())),
            );
            define_func(
                checker,
                &format!("{module}.Encode{root}"),
                vec![p("Method", Ty::String), p("Payload", payload)],
                Ty::Result(Box::new(raw.clone()), Box::new(error.clone())),
            );
        }
    }
}
