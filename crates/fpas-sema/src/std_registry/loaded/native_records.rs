//! Register shared native record declarations and their nested record dependencies.

use crate::{check::Checker, types::Ty};
use fpas_std::{native_errors::ErrorFieldType, native_records};

pub(super) fn register(checker: &mut Checker, name: &str) -> Ty {
    if let Some(symbol) = checker.scopes.lookup(name) {
        return symbol.ty.clone();
    }
    let declaration = native_records::find(name).expect("registered native record");
    let fields = declaration
        .fields
        .iter()
        .map(|(name, shape)| {
            let ty = match shape {
                ErrorFieldType::String => Ty::String,
                ErrorFieldType::Integer => Ty::Integer,
                ErrorFieldType::Strings => Ty::Array(Box::new(Ty::String)),
                ErrorFieldType::OptionalString => Ty::Option(Box::new(Ty::String)),
                ErrorFieldType::OptionalInteger => Ty::Option(Box::new(Ty::Integer)),
                ErrorFieldType::Record(name) => register(checker, name),
                ErrorFieldType::OptionalRecord(name) => {
                    Ty::Option(Box::new(register(checker, name)))
                }
                ErrorFieldType::Host => super::errors::register(
                    checker,
                    fpas_std::native_errors::NativeErrorFamily::Host,
                ),
                ErrorFieldType::Net => super::errors::register(
                    checker,
                    fpas_std::native_errors::NativeErrorFamily::Net,
                ),
            };
            (name.to_string(), ty)
        })
        .collect();
    super::type_registration::register_record_type(checker, declaration.name, fields)
}
