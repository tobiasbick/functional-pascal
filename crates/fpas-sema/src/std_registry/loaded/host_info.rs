//! Typed read-only host-information declarations.
use super::super::define_func;
use crate::{check::Checker, types::Ty};
use fpas_builtins::{
    host_info::{self, Shape},
    native_errors::NativeErrorFamily,
};

pub(super) fn register(checker: &mut Checker, unit: &str) {
    for record in host_info::RECORDS {
        if record
            .name
            .rsplit_once('.')
            .is_some_and(|(owner, _)| owner == unit)
        {
            record_type(checker, record.name);
        }
    }
    let error = super::errors::register(checker, NativeErrorFamily::SystemInfo);
    for (name, output) in host_info::QUERIES {
        if name
            .rsplit_once('.')
            .is_some_and(|(owner, _)| owner == unit)
        {
            let output = shape_type(checker, *output);
            define_func(
                checker,
                name,
                vec![],
                Ty::Result(Box::new(output), Box::new(error.clone())),
            );
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "Names and dependencies come from the static host-information catalog"
)]
fn record_type(checker: &mut Checker, name: &str) -> Ty {
    if let Some(symbol) = checker.scopes.lookup(name) {
        return symbol.ty.clone();
    }
    let declaration = host_info::record(name).expect("static host record declaration");
    let fields = declaration
        .fields
        .iter()
        .map(|(name, shape)| ((*name).to_owned(), shape_type(checker, *shape)))
        .collect();
    super::type_registration::register_record_type(checker, name, fields)
}

fn shape_type(checker: &mut Checker, shape: Shape) -> Ty {
    match shape {
        Shape::Boolean => Ty::Boolean,
        Shape::Integer => Ty::Integer,
        Shape::Real => Ty::Real,
        Shape::String => Ty::String,
        Shape::Option(inner) => Ty::Option(Box::new(shape_type(checker, *inner))),
        Shape::Array(inner) => Ty::Array(Box::new(shape_type(checker, *inner))),
        Shape::Record(name) => record_type(checker, name),
        Shape::Result(inner) => {
            let value = shape_type(checker, *inner);
            let error = super::errors::register(checker, NativeErrorFamily::SystemInfo);
            Ty::Result(Box::new(value), Box::new(error))
        }
    }
}
