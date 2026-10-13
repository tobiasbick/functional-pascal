//! Built-in result record returned by value-producing array Pop.
//! Documentation: `docs/pascal/language/types/array/mutating.md`.

use super::{GenericParamDef, RecordTy, Ty};
use std::sync::Arc;

/// The import-free generic declaration `ArrayPopResult of T`.
#[must_use]
pub fn array_pop_result_declaration() -> Ty {
    let element = Ty::GenericParam("T".into(), None);
    Ty::Record(Arc::new(RecordTy {
        name: "ArrayPopResult".into(),
        owner_unit: None,
        type_params: vec![GenericParamDef {
            name: "T".into(),
            constraint: None,
        }],
        type_args: vec![],
        private_members: vec![],
        fields: vec![
            ("Remaining".into(), Ty::Array(Box::new(element.clone()))),
            ("Value".into(), element),
        ],
        methods: vec![],
        static_functions: vec![],
        static_procedures: vec![],
    }))
}

/// Specializes the result record for an array's element type.
#[must_use]
pub fn array_pop_result(element: Ty) -> Ty {
    let Ty::Record(record) = array_pop_result_declaration() else {
        unreachable!()
    };
    Ty::Record(Arc::new(record.instantiate(vec![element])))
}
