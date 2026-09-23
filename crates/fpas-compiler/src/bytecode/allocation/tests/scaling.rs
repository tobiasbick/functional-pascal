//! Allocation traffic as locals, instruction count, and live values vary independently.
use super::*;

#[test]
fn allocation_traffic_scales_without_rebuilding_local_occupancy() {
    for (locals, values, live) in [
        (16, 512, 1),
        (512, 512, 1),
        (1024, 512, 1),
        (512, 1024, 1),
        (512, 512, 64),
        (512, 512, 128),
    ] {
        let mut function = function_with(Vec::new());
        function.locals = (0..locals)
            .map(|id| Local {
                id: LocalId::new(id),
                ty: TypeId::new(0),
                mutable: true,
                capture: None,
            })
            .collect();
        for base in (0..values).step_by(live as usize) {
            for id in base..(base + live).min(values) {
                function.blocks[0].instructions.push(Instruction {
                    source: None,
                    result: Some(ValueDefinition {
                        id: ValueId::new(id),
                        ty: TypeId::new(0),
                    }),
                    operation: Operation::Const(Constant::Integer(i64::from(id))),
                });
            }
            for id in base..(base + live).min(values) {
                function.blocks[0].instructions.push(Instruction {
                    source: None,
                    result: None,
                    operation: Operation::StoreGlobal {
                        global: GlobalId::new(0),
                        value: ValueId::new(id),
                    },
                });
            }
        }
        let mut result = None;
        let measured = allocation_counter::measure(|| {
            result = Some(Allocation::build(&function));
        });
        let allocation = result.expect("allocation ran").expect("valid IR");
        assert_eq!(allocation.register_count, (locals + live) as u16);
        println!(
            "locals={locals}, values={values}, live={live}, allocations={}, bytes={}",
            measured.count_total, measured.bytes_total
        );
        // Budget proportional to input size; rebuilding all locals for every value exceeds it.
        assert!(
            measured.bytes_total < u64::from(locals + values) * 256,
            "{measured:?}"
        );
    }
}
