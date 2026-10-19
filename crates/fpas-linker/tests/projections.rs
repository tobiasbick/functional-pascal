//! Projection-only concrete types survive symbolic linking and preserve names.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "exact projection link fixtures"
)]

mod common;

use fpas_bytecode::{DebugProjectionKind, DebugType};
use fpas_linker::link_objects;
use fpas_unit::object::{
    DefinitionTarget, ObjectDebugProjection, ObjectDebugProjectionKind, ObjectDebugType,
    ObjectDefinition, ObjectGlobal, ObjectRecordLayout,
};

fn inputs() -> (
    fpas_unit::object::RelocatableObject,
    fpas_unit::object::RelocatableObject,
) {
    let mut library = common::unit(true);
    library.records.push(ObjectRecordLayout {
        name: "library.unit.point".into(),
        fields: vec!["amounts".into()],
        field_types: vec![0],
        field_type_names: vec![None],
        methods: vec![],
        construction: None,
    });
    library.definitions.push(ObjectDefinition {
        name: "library.unit.point".into(),
        target: DefinitionTarget::Record(0),
        public: true,
    });
    library
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));
    let mut root = common::program();
    root.debug_types.extend([
        ObjectDebugType::Integer,
        ObjectDebugType::Record("library.unit.point".into()),
        ObjectDebugType::Array(1),
    ]);
    root.globals.push(ObjectGlobal {
        name: "demo.selected".into(),
        ty: 0,
        type_name: None,
        mutable: true,
        initializer: None,
        projection: Some(ObjectDebugProjection {
            ty: 2,
            type_name: None,
            kind: ObjectDebugProjectionKind::Record(vec![ObjectDebugProjection {
                ty: 3,
                type_name: None,
                kind: ObjectDebugProjectionKind::Array(Box::new(ObjectDebugProjection {
                    ty: 1,
                    type_name: Some("Dollars".into()),
                    kind: ObjectDebugProjectionKind::Scalar,
                })),
            }]),
        }),
    });
    root.definitions.push(ObjectDefinition {
        name: "demo.selected".into(),
        target: DefinitionTarget::Global(0),
        public: false,
    });
    root.definitions
        .sort_by(|left, right| left.name.cmp(&right.name));
    (library, root)
}

#[test]
fn projection_only_imported_layout_and_nominal_array_leaf_are_relocated() {
    let (library, root) = inputs();
    let linked = link_objects(&[library], &root).expect("symbolic projection link");
    let image = linked.executable();
    let projection = image.globals[0]
        .projection
        .as_ref()
        .expect("retained projection");
    assert!(matches!(
        image.debug_types[projection.ty.get() as usize],
        DebugType::Record(_)
    ));
    let DebugProjectionKind::Record(fields) = &projection.kind else {
        panic!("record projection");
    };
    let DebugProjectionKind::Array(element) = &fields[0].kind else {
        panic!("array projection");
    };
    assert_eq!(
        image.strings.get(element.type_name.expect("nominal name")),
        Some("Dollars")
    );
    assert_eq!(
        image.debug_types[element.ty.get() as usize],
        DebugType::Integer
    );
    assert_eq!(
        image.debug_types[image.records[0].fields[0].ty.get() as usize],
        DebugType::Dynamic
    );
}

#[test]
fn imported_projection_field_shape_is_checked_after_layout_resolution() {
    let (library, mut root) = inputs();
    root.globals[0]
        .projection
        .as_mut()
        .expect("projection")
        .kind = ObjectDebugProjectionKind::Record(vec![]);
    assert!(
        root.validate().is_ok(),
        "symbolic layout shape checked after linking"
    );
    assert!(link_objects(&[library], &root).is_err());
}

#[test]
fn operational_tail_accepts_equivalent_types_relocated_from_different_objects() {
    let (mut library, mut root) = inputs();
    library.debug_types.push(ObjectDebugType::Integer);
    library.records[0].field_types[0] = 1;
    root.globals[0]
        .projection
        .as_mut()
        .expect("projection")
        .kind = ObjectDebugProjectionKind::Record(vec![ObjectDebugProjection {
        ty: 1,
        type_name: None,
        kind: ObjectDebugProjectionKind::Operational,
    }]);
    let linked =
        link_objects(&[library], &root).expect("equivalent operational types survive relocation");
    let image = linked.executable();
    let DebugProjectionKind::Record(fields) = &image.globals[0]
        .projection
        .as_ref()
        .expect("projection")
        .kind
    else {
        panic!("record projection");
    };
    assert_eq!(fields[0].kind, DebugProjectionKind::Operational);
    assert_eq!(
        fields[0].ty, image.records[0].fields[0].ty,
        "equivalent object-local types rebase to the defining field's canonical ID"
    );
}

#[test]
fn erased_imported_terminals_rebase_to_the_defining_layout_without_changing_eligibility() {
    for kind in [
        ObjectDebugProjectionKind::Unsupported,
        ObjectDebugProjectionKind::Operational,
    ] {
        let (mut library, mut root) = inputs();
        library.debug_types.push(ObjectDebugType::Integer);
        library.records[0].field_types[0] = 1;
        root.globals[0]
            .projection
            .as_mut()
            .expect("projection")
            .kind = ObjectDebugProjectionKind::Record(vec![ObjectDebugProjection {
            ty: 0,
            type_name: None,
            kind: kind.clone(),
        }]);
        let linked = link_objects(&[library], &root)
            .expect("erased imported terminal matches defining field");
        let image = linked.executable();
        let DebugProjectionKind::Record(fields) = &image.globals[0]
            .projection
            .as_ref()
            .expect("projection")
            .kind
        else {
            panic!("record projection");
        };
        assert_eq!(fields[0].ty, image.records[0].fields[0].ty);
        assert_eq!(
            image.debug_types[fields[0].ty.get() as usize],
            DebugType::Integer
        );
        assert_eq!(
            fields[0].kind,
            if kind == ObjectDebugProjectionKind::Unsupported {
                DebugProjectionKind::Unsupported
            } else {
                DebugProjectionKind::Operational
            }
        );
    }
}

#[test]
fn terminal_rebase_does_not_hide_known_storage_type_contradictions() {
    for kind in [
        ObjectDebugProjectionKind::Unsupported,
        ObjectDebugProjectionKind::Operational,
    ] {
        let (mut library, mut root) = inputs();
        library.debug_types.push(ObjectDebugType::Integer);
        library.records[0].field_types[0] = 1;
        root.debug_types.push(ObjectDebugType::String);
        root.globals[0]
            .projection
            .as_mut()
            .expect("projection")
            .kind = ObjectDebugProjectionKind::Record(vec![ObjectDebugProjection {
            ty: 4,
            type_name: None,
            kind,
        }]);
        assert!(
            link_objects(&[library], &root).is_err(),
            "known String/Integer contradiction remains invalid"
        );
    }
}

#[test]
fn imported_array_erasure_rebases_container_and_retains_unsupported_leaf() {
    let (mut library, mut root) = inputs();
    library
        .debug_types
        .extend([ObjectDebugType::Integer, ObjectDebugType::Array(1)]);
    library.records[0].field_types[0] = 2;
    root.debug_types[3] = ObjectDebugType::Array(0);
    root.globals[0]
        .projection
        .as_mut()
        .expect("projection")
        .kind = ObjectDebugProjectionKind::Record(vec![ObjectDebugProjection {
        ty: 3,
        type_name: None,
        kind: ObjectDebugProjectionKind::Array(Box::new(ObjectDebugProjection {
            ty: 0,
            type_name: None,
            kind: ObjectDebugProjectionKind::Unsupported,
        })),
    }]);
    let linked =
        link_objects(&[library], &root).expect("imported array erasure matches defining field");
    let image = linked.executable();
    let DebugProjectionKind::Record(fields) = &image.globals[0]
        .projection
        .as_ref()
        .expect("projection")
        .kind
    else {
        panic!("record projection");
    };
    assert_eq!(fields[0].ty, image.records[0].fields[0].ty);
    let DebugProjectionKind::Array(child) = &fields[0].kind else {
        panic!("array projection");
    };
    assert_eq!(child.kind, DebugProjectionKind::Unsupported);
    assert_eq!(
        image.debug_types[child.ty.get() as usize],
        DebugType::Integer
    );
}

#[test]
fn container_rebase_preserves_concrete_refinements_and_rejects_known_contradictions() {
    for reference in [false, true] {
        for case in 0..3 {
            let container = |inner| {
                if reference {
                    ObjectDebugType::Reference(inner)
                } else {
                    ObjectDebugType::Array(inner)
                }
            };
            let (mut library, mut root) = inputs();
            library
                .debug_types
                .extend([ObjectDebugType::Integer, container(1)]);
            library.records[0].field_types[0] = if case == 1 { 0 } else { 2 };
            root.debug_types.extend([
                ObjectDebugType::String,
                container(0),
                container(1),
                container(4),
            ]);
            let child = ObjectDebugProjection {
                ty: if case == 1 { 1 } else { 4 },
                type_name: None,
                kind: if case == 2 {
                    ObjectDebugProjectionKind::Unsupported
                } else {
                    ObjectDebugProjectionKind::Scalar
                },
            };
            let projection = ObjectDebugProjection {
                ty: if case == 0 {
                    5
                } else if case == 1 {
                    6
                } else {
                    7
                },
                type_name: None,
                kind: if reference {
                    ObjectDebugProjectionKind::Reference(Box::new(child))
                } else {
                    ObjectDebugProjectionKind::Array(Box::new(child))
                },
            };
            root.globals[0]
                .projection
                .as_mut()
                .expect("projection")
                .kind = ObjectDebugProjectionKind::Record(vec![projection]);
            let linked = link_objects(&[library], &root);
            assert_eq!(
                linked.is_ok(),
                case == 1,
                "reference={reference}, case={case}"
            );
            if let Ok(linked) = linked {
                let image = linked.executable();
                let DebugProjectionKind::Record(fields) = &image.globals[0]
                    .projection
                    .as_ref()
                    .expect("projection")
                    .kind
                else {
                    panic!("record projection");
                };
                let child = match &fields[0].kind {
                    DebugProjectionKind::Array(child) | DebugProjectionKind::Reference(child) => {
                        child
                    }
                    _ => panic!("container projection"),
                };
                assert_eq!(child.kind, DebugProjectionKind::Scalar);
                assert_eq!(
                    image.debug_types[child.ty.get() as usize],
                    DebugType::Integer,
                    "concrete refinement is retained when canonical storage is Dynamic"
                );
            }
        }
    }
}
