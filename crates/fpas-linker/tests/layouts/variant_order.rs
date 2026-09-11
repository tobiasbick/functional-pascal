//! Variant relocation must follow the emitted canonical layout order.

use super::*;

#[test]
fn constructors_and_patterns_keep_variant_identity_after_layout_sorting() {
    let mut library = common::unit(true);
    for name in ["shared.zed", "shared.alpha"] {
        let index = u32::try_from(library.enums.len()).expect("layout index");
        library.enums.push(fpas_unit::object::ObjectEnumLayout {
            name: name.to_string(),
            variants: ["first", "second"]
                .into_iter()
                .map(|variant| fpas_unit::object::ObjectEnumVariant {
                    name: variant.to_string(),
                    fields: Vec::new(),
                    field_types: Vec::new(),
                })
                .collect(),
        });
        library.definitions.push(ObjectDefinition {
            name: name.to_string(),
            target: DefinitionTarget::Enum(index),
            public: true,
        });
    }
    library.definitions.sort_by(|a, b| a.name.cmp(&b.name));
    let mut root = common::program();
    root.imports = vec![ObjectImport {
        name: "shared.zed".to_string(),
        shape: ImportShape::Enum {
            variants: vec![
                ("first".to_string(), vec![]),
                ("second".to_string(), vec![]),
            ],
        },
    }];
    root.functions[0].register_count = 2;
    root.functions[0].code = vec![
        Instruction::abc(Opcode::MakeEnum, 0, 0, 0, 0)
            .expect("constructor")
            .word(),
        Instruction::abc(Opcode::TestVariant, 1, 0, 0, 0)
            .expect("pattern")
            .word(),
        common::return_unit(),
    ];
    root.relocations = (0..2)
        .map(|instruction| Relocation {
            function: 0,
            instruction,
            kind: RelocationKind::EnumVariant {
                enumeration: SymbolReference::Import(0),
                variant: "second".to_string(),
            },
        })
        .collect();

    let linked = link_objects(&[library], &root).expect("link");
    let executable = linked.executable();
    let constructor = executable.code[0]
        .abc_operands()
        .expect("constructor operands")
        .b;
    let pattern = executable.code[1]
        .abc_operands()
        .expect("pattern operands")
        .c;
    for id in [constructor, pattern] {
        let variant = &executable.enum_variants[usize::from(id)];
        assert_eq!(executable.strings.get(variant.name), Some("second"));
        assert_eq!(
            executable
                .strings
                .get(executable.enums[usize::from(variant.owner.get())].name),
            Some("shared.zed")
        );
    }
}
