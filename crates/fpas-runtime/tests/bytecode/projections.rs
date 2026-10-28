//! Source projections refine erased fields without changing stored layouts.

use super::support::minimal_executable;
use fpas_runtime::{
    DebugProjection, DebugProjectionKind, DebugType, DebugTypeId, GlobalInfo, RecordField,
    RecordLayout, RecordTypeId, StringId, ValidationErrorKind,
};

fn scalar(ty: u32) -> DebugProjection {
    DebugProjection {
        ty: DebugTypeId::new(ty),
        type_name: Some(StringId::new(0)),
        kind: DebugProjectionKind::Scalar,
    }
}

fn candidate() -> fpas_runtime::Executable {
    let mut image = minimal_executable();
    image.debug_types = vec![
        DebugType::Dynamic,
        DebugType::Integer,
        DebugType::String,
        DebugType::Record(RecordTypeId::new(0)),
        DebugType::Array(DebugTypeId::new(0)),
        DebugType::Array(DebugTypeId::new(1)),
    ];
    image.records.push(RecordLayout {
        name: StringId::new(0),
        fields: vec![RecordField {
            name: StringId::new(0),
            ty: DebugTypeId::new(0),
            type_name: None,
        }],
        methods: vec![],
        construction: None,
    });
    image.globals.push(GlobalInfo {
        name: StringId::new(0),
        ty: DebugTypeId::new(3),
        type_name: None,
        mutable: true,
        initializer: None,
        projection: Some(DebugProjection {
            ty: DebugTypeId::new(3),
            type_name: None,
            kind: DebugProjectionKind::Record(vec![scalar(1)]),
        }),
    });
    image
}

#[test]
fn different_generic_applications_share_layout_without_sharing_projection() {
    let mut image = candidate();
    let mut other = image.globals[0].clone();
    other.projection.as_mut().expect("projection").kind =
        DebugProjectionKind::Record(vec![scalar(2)]);
    image.globals.push(other);
    let verified = image
        .verify()
        .expect("concrete projections refine erased fields");
    assert_ne!(
        verified.executable().globals[0].projection,
        verified.executable().globals[1].projection
    );
    assert_eq!(
        verified.executable().records[0].fields[0].ty,
        DebugTypeId::new(0)
    );
}

#[test]
fn nested_array_projection_may_refine_erased_elements() {
    let mut image = candidate();
    image.globals[0].ty = DebugTypeId::new(4);
    image.globals[0].projection = Some(DebugProjection {
        ty: DebugTypeId::new(5),
        type_name: None,
        kind: DebugProjectionKind::Array(Box::new(scalar(1))),
    });
    image.verify().expect("compatible refined array");
}

#[test]
fn malformed_projection_types_shapes_and_names_are_rejected() {
    for failure in 0..6 {
        let mut image = candidate();
        let projection = image.globals[0].projection.as_mut().expect("projection");
        match failure {
            0 => projection.ty = DebugTypeId::new(1),
            1 => projection.kind = DebugProjectionKind::Record(vec![]),
            2 => projection.kind = DebugProjectionKind::Scalar,
            3 => projection.type_name = Some(StringId::new(u32::MAX)),
            4 => projection.kind = DebugProjectionKind::Record(vec![scalar(u32::MAX)]),
            _ => {
                image.records[0].fields[0].ty = DebugTypeId::new(1);
                projection.kind = DebugProjectionKind::Record(vec![scalar(2)]);
            }
        }
        assert!(matches!(
            image.verify().expect_err("invalid projection").kind,
            ValidationErrorKind::DebugProjection { .. }
        ));
    }
}

#[test]
fn projection_tree_depth_and_name_byte_limits_are_enforced() {
    for depth in [32, 33] {
        let mut image = minimal_executable();
        image.debug_types = vec![DebugType::Dynamic];
        let mut node = DebugProjection {
            ty: DebugTypeId::new(0),
            type_name: None,
            kind: DebugProjectionKind::Unsupported,
        };
        for _ in 0..depth {
            let id = DebugTypeId::new(image.debug_types.len() as u32);
            image.debug_types.push(DebugType::Array(node.ty));
            node = DebugProjection {
                ty: id,
                type_name: None,
                kind: DebugProjectionKind::Array(Box::new(node)),
            };
        }
        image.globals.push(GlobalInfo {
            projection: Some(node.clone()),
            name: StringId::new(0),
            ty: node.ty,
            type_name: None,
            mutable: true,
            initializer: None,
        });
        assert_eq!(image.verify().is_ok(), depth == 32);
    }
    let mut image = candidate();
    image.strings = fpas_runtime::StringTable::new(vec![
        "x".repeat(fpas_runtime::MAX_DEBUG_PROJECTION_NAME_BYTES + 1),
        "test.fpas".into(),
    ]);
    assert!(image.verify().is_err());
}

#[test]
fn operational_tails_match_storage_without_refining_dynamic_or_adding_names() {
    for (stored, projected, named, valid) in [
        (1, 1, false, true),
        (0, 0, false, true),
        (0, 1, false, false),
        (4, 5, false, false),
        (4, 6, false, true),
        (1, 1, true, false),
    ] {
        let mut image = candidate();
        image
            .debug_types
            .push(DebugType::Array(DebugTypeId::new(0)));
        image.globals[0].ty = DebugTypeId::new(stored);
        image.globals[0].projection = Some(DebugProjection {
            ty: DebugTypeId::new(projected),
            type_name: named.then_some(StringId::new(0)),
            kind: DebugProjectionKind::Operational,
        });
        assert_eq!(
            image.verify().is_ok(),
            valid,
            "stored={stored}, projected={projected}, named={named}"
        );
    }
}
