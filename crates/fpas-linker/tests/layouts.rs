//! Canonical layout identity and coalescing during linking.

#![allow(
    clippy::expect_used,
    reason = "focused linker fixtures use explicit expectations"
)]

mod common;

use fpas_bytecode::{Instruction, Opcode};
use fpas_linker::{LinkError, link_objects};
use fpas_unit::object::{
    DefinitionTarget, ImportShape, ObjectDefinition, ObjectImport, Relocation, RelocationKind,
    SymbolReference,
};

use common::*;

#[test]
fn matching_private_layout_copies_share_one_canonical_type_id() {
    let mut first = unit(true);
    first.records.push(fpas_unit::object::ObjectRecordLayout {
        name: "std.console.keyevent".to_string(),
        fields: vec!["kind".to_string(), "character".to_string()],
        field_types: vec![0, 0],
        properties: Vec::new(),
        methods: Vec::new(),
    });
    first.definitions.push(ObjectDefinition {
        name: "std.console.keyevent".to_string(),
        target: DefinitionTarget::Record(0),
        public: false,
    });
    first
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));

    let mut second = unit(false);
    second.owner = "other.unit".to_string();
    for function in &mut second.functions {
        function.name = function.name.replacen("library.unit", "other.unit", 1);
    }
    for definition in &mut second.definitions {
        definition.name = definition.name.replacen("library.unit", "other.unit", 1);
    }
    second.records.push(fpas_unit::object::ObjectRecordLayout {
        name: "std.console.keyevent".to_string(),
        fields: vec!["kind".to_string(), "character".to_string()],
        field_types: vec![0, 0],
        properties: Vec::new(),
        methods: Vec::new(),
    });
    second.definitions.push(ObjectDefinition {
        name: "std.console.keyevent".to_string(),
        target: DefinitionTarget::Record(0),
        public: false,
    });
    second
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));

    let executable = link_objects(&[first, second], &program())
        .expect("matching private layout copies must coalesce");
    assert_eq!(executable.executable().records.len(), 1);
}

#[test]
fn record_layout_copies_reject_structurally_different_field_types() {
    let left = private_record_copy(
        "library.unit",
        vec![fpas_unit::object::ObjectDebugType::Integer],
        one_field_record(0),
    );
    let right = private_record_copy(
        "other.unit",
        vec![fpas_unit::object::ObjectDebugType::Real],
        one_field_record(0),
    );

    assert!(matches!(
        link_objects(&[left, right], &program()),
        Err(LinkError::IncompatibleLayoutCopies {
            name,
            left_owner,
            right_owner,
        }) if name == "shared.node"
            && left_owner == "library.unit"
            && right_owner == "other.unit"
    ));
}

#[test]
fn equivalent_field_types_at_different_local_indexes_coalesce() {
    let left = private_record_copy(
        "library.unit",
        vec![fpas_unit::object::ObjectDebugType::Integer],
        one_field_record(0),
    );
    let right = private_record_copy(
        "other.unit",
        vec![
            fpas_unit::object::ObjectDebugType::Dynamic,
            fpas_unit::object::ObjectDebugType::Integer,
        ],
        one_field_record(1),
    );

    let executable =
        link_objects(&[left, right], &program()).expect("equivalent copies must coalesce");
    assert_eq!(executable.executable().records.len(), 1);
}

#[test]
fn recursive_nested_layout_types_terminate_and_coalesce() {
    let left = private_record_copy(
        "library.unit",
        vec![
            fpas_unit::object::ObjectDebugType::Record("shared.node".to_string()),
            fpas_unit::object::ObjectDebugType::Array(0),
        ],
        one_field_record(1),
    );
    let right = private_record_copy(
        "other.unit",
        vec![
            fpas_unit::object::ObjectDebugType::Dynamic,
            fpas_unit::object::ObjectDebugType::Record("shared.node".to_string()),
            fpas_unit::object::ObjectDebugType::Array(1),
        ],
        one_field_record(2),
    );

    let executable =
        link_objects(&[left, right], &program()).expect("recursive copies must coalesce");
    assert_eq!(executable.executable().records.len(), 1);
}

#[test]
fn record_layout_copies_compare_properties_and_methods() {
    let mut left_layout = one_field_record(0);
    left_layout
        .properties
        .push(fpas_unit::object::ObjectRecordProperty {
            name: "Value".to_string(),
            getter: "shared.node.getvalue".to_string(),
        });
    left_layout
        .methods
        .push(fpas_unit::object::ObjectRecordMethod {
            name: "Reset".to_string(),
            routine: "shared.node.reset".to_string(),
        });
    let mut right_layout = left_layout.clone();
    right_layout.properties[0].getter = "shared.node.getother".to_string();
    let left = private_record_copy(
        "library.unit",
        vec![fpas_unit::object::ObjectDebugType::Integer],
        left_layout,
    );
    let right = private_record_copy(
        "other.unit",
        vec![fpas_unit::object::ObjectDebugType::Integer],
        right_layout,
    );

    assert!(matches!(
        link_objects(&[left, right], &program()),
        Err(LinkError::IncompatibleLayoutCopies { name, .. }) if name == "shared.node"
    ));
}

#[test]
fn enum_layout_copies_compare_variant_field_types() {
    let mut left = unit(false);
    left.enums.push(fpas_unit::object::ObjectEnumLayout {
        name: "shared.choice".to_string(),
        variants: vec![fpas_unit::object::ObjectEnumVariant {
            name: "some".to_string(),
            fields: vec!["value".to_string()],
            field_types: vec![0],
        }],
    });
    left.debug_types = vec![fpas_unit::object::ObjectDebugType::Integer];
    left.definitions.push(ObjectDefinition {
        name: "shared.choice".to_string(),
        target: DefinitionTarget::Enum(0),
        public: false,
    });
    left.definitions.sort_by(|a, b| a.name.cmp(&b.name));
    let mut right = left.clone();
    right.owner = "other.unit".to_string();
    for function in &mut right.functions {
        function.name = function.name.replacen("library.unit", "other.unit", 1);
    }
    for definition in &mut right.definitions {
        if definition.name != "shared.choice" {
            definition.name = definition.name.replacen("library.unit", "other.unit", 1);
        }
    }
    right.debug_types = vec![fpas_unit::object::ObjectDebugType::Real];
    right.definitions.sort_by(|a, b| a.name.cmp(&b.name));

    assert!(matches!(
        link_objects(&[left, right], &program()),
        Err(LinkError::IncompatibleLayoutCopies { name, .. }) if name == "shared.choice"
    ));
}

#[test]
fn incompatible_record_layout_import_is_rejected_before_relocation() {
    let mut library = unit(true);
    library.records.push(fpas_unit::object::ObjectRecordLayout {
        name: "library.unit.point".to_string(),
        fields: vec!["x".to_string(), "y".to_string()],
        field_types: vec![0, 0],
        properties: Vec::new(),
        methods: Vec::new(),
    });
    library.definitions.push(ObjectDefinition {
        name: "library.unit.point".to_string(),
        target: DefinitionTarget::Record(0),
        public: true,
    });
    library
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));
    let mut root = program();
    root.imports.push(ObjectImport {
        name: "library.unit.point".to_string(),
        shape: ImportShape::Record {
            fields: vec!["y".to_string(), "x".to_string()],
        },
    });
    root.imports
        .sort_by(|left, right| left.name.cmp(&right.name));
    let function_import = root
        .imports
        .iter()
        .position(|import| import.name == "library.unit.zed")
        .expect("function import index");
    root.relocations[0].kind = RelocationKind::Function(SymbolReference::Import(
        u32::try_from(function_import).expect("import index"),
    ));
    assert!(matches!(
        link_objects(&[library], &root),
        Err(LinkError::IncompatibleImport { .. })
    ));
}

#[test]
fn imported_global_record_and_enum_references_become_dense_numeric_ids() {
    let mut library = unit(true);
    library.globals.push(fpas_unit::object::ObjectGlobal {
        name: "library.unit.counter".to_string(),
        ty: 0,
        mutable: true,
        initializer: None,
    });
    library.records.push(fpas_unit::object::ObjectRecordLayout {
        name: "library.unit.point".to_string(),
        fields: vec!["x".to_string()],
        field_types: vec![0],
        properties: Vec::new(),
        methods: vec![fpas_unit::object::ObjectRecordMethod {
            name: "translate".to_string(),
            routine: "library.unit.alpha".to_string(),
        }],
    });
    library.enums.push(fpas_unit::object::ObjectEnumLayout {
        name: "library.unit.choice".to_string(),
        variants: vec![fpas_unit::object::ObjectEnumVariant {
            name: "some".to_string(),
            fields: vec!["value".to_string()],
            field_types: vec![0],
        }],
    });
    library.debug_types = vec![
        fpas_unit::object::ObjectDebugType::Integer,
        fpas_unit::object::ObjectDebugType::Record("library.unit.point".to_string()),
        fpas_unit::object::ObjectDebugType::Enum("library.unit.choice".to_string()),
        fpas_unit::object::ObjectDebugType::Array(0),
    ];
    library.definitions.extend([
        ObjectDefinition {
            name: "library.unit.choice".to_string(),
            target: DefinitionTarget::Enum(0),
            public: true,
        },
        ObjectDefinition {
            name: "library.unit.counter".to_string(),
            target: DefinitionTarget::Global(0),
            public: true,
        },
        ObjectDefinition {
            name: "library.unit.point".to_string(),
            target: DefinitionTarget::Record(0),
            public: true,
        },
    ]);
    library
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));

    let mut root = program();
    root.functions[0].code = vec![
        Instruction::abx(Opcode::LoadGlobal, 0, 0)
            .expect("global")
            .word(),
        Instruction::abc(Opcode::StoreGlobalIndexPath, 0, 0, 1, 0)
            .expect("global index path")
            .word(),
        Instruction::abc(Opcode::MakeRecord, 1, 0, 0, 0)
            .expect("record")
            .word(),
        Instruction::abc(Opcode::MakeEnum, 2, 0, 0, 0)
            .expect("enum")
            .word(),
        return_unit(),
    ];
    root.functions[0].register_count = 3;
    root.imports = vec![
        ObjectImport {
            name: "library.unit.choice".to_string(),
            shape: ImportShape::Enum {
                variants: vec![("some".to_string(), vec!["value".to_string()])],
            },
        },
        ObjectImport {
            name: "library.unit.counter".to_string(),
            shape: ImportShape::Global { mutable: true },
        },
        ObjectImport {
            name: "library.unit.point".to_string(),
            shape: ImportShape::Record {
                fields: vec!["x".to_string()],
            },
        },
    ];
    root.relocations = vec![
        Relocation {
            function: 0,
            instruction: 0,
            kind: RelocationKind::Global(SymbolReference::Import(1)),
        },
        Relocation {
            function: 0,
            instruction: 1,
            kind: RelocationKind::Global(SymbolReference::Import(1)),
        },
        Relocation {
            function: 0,
            instruction: 2,
            kind: RelocationKind::Record(SymbolReference::Import(2)),
        },
        Relocation {
            function: 0,
            instruction: 3,
            kind: RelocationKind::EnumVariant {
                enumeration: SymbolReference::Import(0),
                variant: "some".to_string(),
            },
        },
    ];
    let linked = link_objects(&[library], &root).expect("layout link");
    assert_eq!(linked.executable().globals.len(), 1);
    assert_eq!(linked.executable().records.len(), 1);
    let method = linked.executable().records[0].methods[0];
    assert_eq!(
        linked.executable().strings.get(method.name),
        Some("translate")
    );
    assert_eq!(
        linked.executable().strings.get(method.routine),
        Some("library.unit.alpha")
    );
    assert_eq!(linked.executable().enums.len(), 1);
    assert_eq!(linked.executable().enum_variants.len(), 1);
    assert_eq!(
        linked.executable().debug_types[..4],
        [
            fpas_bytecode::DebugType::Integer,
            fpas_bytecode::DebugType::Record(fpas_bytecode::RecordTypeId::new(0)),
            fpas_bytecode::DebugType::Enum(fpas_bytecode::EnumTypeId::new(0)),
            fpas_bytecode::DebugType::Array(fpas_bytecode::DebugTypeId::new(0)),
        ]
    );
    assert_eq!(linked.executable().globals[0].ty.get(), 0);
    assert_eq!(linked.executable().records[0].fields[0].ty.get(), 0);
    assert_eq!(linked.executable().enum_variants[0].field_types[0].get(), 0);
    assert_eq!(
        linked.executable().code[0]
            .abx_operands()
            .expect("global operands")
            .bx,
        0
    );
    assert_eq!(
        linked.executable().code[1]
            .abc_operands()
            .expect("global index path operands")
            .b,
        0
    );
    assert_eq!(
        linked.executable().code[2]
            .abc_operands()
            .expect("record operands")
            .b,
        0
    );
    assert_eq!(
        linked.executable().code[3]
            .abc_operands()
            .expect("enum operands")
            .b,
        0
    );
}
