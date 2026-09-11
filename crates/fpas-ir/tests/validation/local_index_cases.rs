#[test]
fn local_index_store_validates_mutability_types_operands_and_result_shape() {
    let mut valid = all_operations_program();
    valid.functions[0].blocks[0].instructions.push(Instruction {
        source: None,
        result: None,
        operation: Operation::StoreLocalIndex {
            local: LocalId::new(1),
            index: ValueId::new(1),
            value: ValueId::new(1),
        },
    });
    assert!(valid.validate().is_ok());
    for (local, index, replacement) in [
        (99, 1, 1),
        (0, 1, 1),
        (1, 3, 1),
        (1, 1, 3),
        (1, 99, 1),
        (1, 1, 99),
    ] {
        let mut invalid = valid.clone();
        let last = invalid.functions[0].blocks[0].instructions.len() - 1;
        invalid.functions[0].blocks[0].instructions[last].operation = Operation::StoreLocalIndex {
            local: LocalId::new(local),
            index: ValueId::new(index),
            value: ValueId::new(replacement),
        };
        assert!(
            invalid.validate().is_err(),
            "local={local}, index={index}, value={replacement}"
        );
    }
    let mut immutable = valid.clone();
    immutable.functions[0].locals[1].mutable = false;
    assert!(immutable.validate().is_err());
    let mut result = valid;
    let last = result.functions[0].blocks[0].instructions.len() - 1;
    result.functions[0].blocks[0].instructions[last].result = Some(value(999, ARRAY));
    assert!(result.validate().is_err());
}

#[test]
fn local_dictionary_store_requires_declared_key_and_value_types() {
    let mut program = all_operations_program();
    program.functions[0].locals.push(Local {
        id: LocalId::new(2),
        ty: DICTIONARY,
        mutable: true,
        capture: None,
    });
    let last = program.functions[0].blocks[0].instructions.len();
    program.functions[0].blocks[0]
        .instructions
        .push(Instruction {
            source: None,
            result: None,
            operation: Operation::StoreLocalIndex {
                local: LocalId::new(2),
                index: ValueId::new(3),
                value: ValueId::new(1),
            },
        });
    assert!(program.validate().is_ok());
    for (index, value) in [(1, 1), (3, 3)] {
        program.functions[0].blocks[0].instructions[last].operation = Operation::StoreLocalIndex {
            local: LocalId::new(2),
            index: ValueId::new(index),
            value: ValueId::new(value),
        };
        assert!(program.validate().is_err());
    }
}
