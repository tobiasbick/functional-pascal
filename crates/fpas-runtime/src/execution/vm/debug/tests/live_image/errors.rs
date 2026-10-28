//! Essential error metadata participates in live-image compatibility.

use super::*;

#[test]
fn changed_error_identity_or_literal_label_rejects_live_replacement() {
    let mut image = pair_executable().into_unverified();
    let mut strings = image.strings.iter().map(str::to_owned).collect::<Vec<_>>();
    let base = strings.len();
    strings.extend(
        [
            "Example.Failure",
            "Failed",
            "Original error label",
            "Changed error label",
        ]
        .map(str::to_owned),
    );
    image.strings = crate::StringTable::new(strings);
    let id = |offset| StringId::try_from_index(base + offset).expect("fixture string");
    image.enums.push(EnumLayout {
        name: id(0),
        error: Some(crate::ErrorInstanceKey {
            declaration: "example.failure".to_owned(),
            arguments: vec![],
        }),
    });
    image.enum_variants.push(crate::EnumVariant {
        owner: crate::EnumTypeId::new(0),
        name: id(1),
        fields: vec![],
        field_types: vec![],
        error: Some(crate::ErrorVariant {
            message: Some(id(2)),
            cause: None,
            field_types: vec![],
        }),
    });
    let current = image.verify().expect("error image");
    for identity_change in [false, true] {
        let mut candidate = current.clone().into_unverified();
        if identity_change {
            candidate.enums[0]
                .error
                .as_mut()
                .expect("error instance")
                .arguments
                .push(crate::TypeIdentity::Integer);
        } else {
            candidate.enum_variants[0]
                .error
                .as_mut()
                .expect("error variant")
                .message = Some(id(3));
        }
        let result = classify_current(
            current.clone(),
            candidate.verify().expect("consistent candidate"),
        );
        assert_eq!(result.class, LiveImageUpdateClass::EnumLayout);
        assert!(!result.accepted);
    }
}
