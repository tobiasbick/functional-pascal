//! Literal protocol examples cover every SDK client-role method in both directions.

#[path = "methods/v1.rs"]
mod v1;
#[path = "methods/v2.rs"]
mod v2;

use fpas_acp::{
    RawJson,
    binding::{self, Shape, Type, Value},
    schema::Version,
};
use std::collections::BTreeSet;

fn check(version: Version, fixtures: &[(&str, &str, &str)]) {
    let catalog = binding::catalog(version).unwrap();
    let mut covered = BTreeSet::new();
    for &(root, method, text) in fixtures {
        let decoded = catalog
            .decode_message(root, method, RawJson::parse(text).unwrap())
            .unwrap_or_else(|error| panic!("{version:?} {root} {method}: {error}"));
        let Value::Enum { variant, .. } = &decoded.payload else {
            panic!("root enum");
        };
        covered.insert((root.to_owned(), variant.clone()));
        catalog
            .encode_message(root, method, &decoded.payload)
            .unwrap_or_else(|error| panic!("encode {version:?} {root} {method}: {error}"));
    }
    for root in binding::ROOTS {
        let Shape::Enum { variants, .. } = &catalog.declarations[root].shape else {
            panic!("root declaration");
        };
        for variant in variants {
            let Some(Type::Named(key)) = &variant.payload else {
                panic!("named payload");
            };
            assert!(
                covered.contains(&(root.into(), variant.name.clone())),
                "missing independent {version:?} fixture: {root} {key}"
            );
        }
    }
}

#[test]
fn all_v1_method_payloads_have_independent_wire_examples() {
    check(Version::V1, v1::FIXTURES);
}

#[test]
fn all_v2_method_payloads_have_independent_wire_examples() {
    check(Version::V2, v2::FIXTURES);
}
