//! Balance aggregate backing storage as well as cycle collector cell counters.
use crate::measurement::completed_threads;
use crate::workloads::{function, image};
use fpas_bytecode::{
    CollectionLimits, DomainMaintenance, EnumTypeId, EnumVariantId, FunctionTarget,
    MaintenanceOutcome, MaintenancePolicy, MemoryDomain, OwnershipError, RecordTypeId, RootKind,
    RuntimeEnumLayout, RuntimeRecordLayout, SharedEnum, SharedRecord, Value,
};
use std::sync::Arc;

struct Collector;

impl DomainMaintenance for Collector {
    fn maintain(&self, domain: &MemoryDomain, roots: &[Value]) -> Result<(), OwnershipError> {
        domain
            .reclaim_unreachable(roots, CollectionLimits::default())
            .map(|_| ())
    }
}

fn collect(domain: &MemoryDomain) {
    assert!(matches!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Serviced(_)
    ));
}

fn payload(kind: usize, next: Value, target: &FunctionTarget) -> Value {
    // Oversized buffers must be deallocated; smaller wrappers exercise reusable pools.
    let data = Value::Array(vec![Value::Str("x".repeat(64 * 1024).into()); 8192].into());
    let pair = Value::Array(vec![next, data].into());
    match kind {
        0 => pair,
        1 => Value::dict(vec![(Value::Integer(1), pair)]),
        2 => Value::dict(vec![(pair, Value::Integer(1))]),
        3 => Value::Record(SharedRecord::new(
            Arc::new(RuntimeRecordLayout {
                record: RecordTypeId::new(0),
                type_name: "Payload".into(),
                fields: vec!["Data".into()],
            }),
            vec![pair],
        )),
        4 => Value::Enum(SharedEnum::new(
            Arc::new(RuntimeEnumLayout {
                error: None,
                enumeration: EnumTypeId::new(0),
                variant_id: EnumVariantId::new(0),
                type_name: "Payload".into(),
                variant: "Data".into(),
                fields: vec!["Data".into()],
            }),
            vec![pair],
        )),
        5 => Value::option_some(pair),
        6 => Value::result_ok(pair),
        7 => Value::result_error(pair),
        8 => Value::function(target.clone(), vec![pair]),
        9 => Value::bound_function(target.clone(), pair),
        _ => unreachable!(),
    }
}

pub fn collect_payloads() {
    let image = image("program Payloads; procedure Target(); begin end procedure; begin end.");
    let target = image.function_target(function(&image, "Target")).unwrap();
    completed_threads("aggregate domain teardown", 8, || {
        let domain = MemoryDomain::new();
        domain.set_maintenance(Arc::new(Collector), MaintenancePolicy::Explicit);
        for _ in 0..8 {
            for kind in 0..10 {
                let a = domain.new_cell(Value::Unit).unwrap();
                let b = domain.new_cell(Value::Unit).unwrap();
                let wa = a.downgrade();
                let wb = b.downgrade();
                a.replace(&domain, payload(kind, Value::Cell(b.clone()), &target))
                    .unwrap();
                b.replace(
                    &domain,
                    Value::function(target.clone(), vec![Value::Cell(a.clone())]),
                )
                .unwrap();
                let root = domain.root_values(RootKind::Host, vec![Value::Cell(a)]);
                drop(b);
                collect(&domain);
                assert!(wa.upgrade().is_some() && wb.upgrade().is_some());
                drop(root);
                collect(&domain);
                assert!(wa.upgrade().is_none() && wb.upgrade().is_none());
                let stats = domain.collection_stats();
                assert_eq!(stats.live_cells, 0);
                assert_eq!(stats.allocated_cells, stats.released_cells);
            }
        }
    });
}
