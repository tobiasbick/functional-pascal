use super::*;
use crate::value::graph::tests::target;
use crate::{
    EnumTypeId, EnumVariantId, RecordTypeId, RuntimeEnumLayout, RuntimeRecordLayout, SharedEnum,
    SharedRecord,
};

fn edge(kind: usize, next: Value) -> Value {
    match kind {
        0 => Value::Array(vec![next].into()),
        1 => Value::dict(vec![(Value::Integer(1), next)]),
        2 => Value::dict(vec![(next, Value::Integer(2))]),
        3 => Value::Record(SharedRecord::new(
            Arc::new(RuntimeRecordLayout {
                record: RecordTypeId::new(0),
                type_name: "Edge".into(),
                fields: vec!["Next".into()],
            }),
            vec![next],
        )),
        4 => Value::Enum(SharedEnum::new(
            Arc::new(RuntimeEnumLayout {
                error: None,
                enumeration: EnumTypeId::new(0),
                variant_id: EnumVariantId::new(0),
                type_name: "Edge".into(),
                variant: "Next".into(),
                fields: vec!["Value".into()],
            }),
            vec![next],
        )),
        5 => Value::option_some(next),
        6 => Value::result_ok(next),
        7 => Value::result_error(next),
        8 => Value::function(target(), vec![next]),
        9 => Value::bound_function(target(), next),
        _ => unreachable!(),
    }
}
#[test]
fn every_aggregate_edge_participates_in_marking_and_mutual_cycle_reclamation() {
    for kind in 0..10 {
        let domain = domain();
        let a = domain.new_cell(Value::Unit).unwrap();
        let b = domain.new_cell(Value::Unit).unwrap();
        let wa = a.downgrade();
        let wb = b.downgrade();
        a.replace(&domain, edge(kind, Value::Cell(b.clone())))
            .unwrap();
        b.replace(
            &domain,
            Value::function(target(), vec![Value::Cell(a.clone())]),
        )
        .unwrap();
        let rooted = domain.root_values(RootKind::Host, vec![Value::Cell(a)]);
        drop(b);
        collect(&domain);
        assert!(wa.upgrade().is_some());
        assert!(wb.upgrade().is_some());
        drop(rooted);
        collect(&domain);
        assert!(wa.upgrade().is_none(), "edge {kind}");
        assert!(wb.upgrade().is_none(), "edge {kind}");
        assert_eq!(domain.collection_stats().released_cells, 2);
    }
}
#[test]
fn self_referential_callable_is_reclaimed_without_weakening_capture_edges() {
    let domain = domain();
    let cell = domain.new_cell(Value::Unit).unwrap();
    let weak = cell.downgrade();
    cell.replace(
        &domain,
        Value::task_owned_function(
            target(),
            vec![Value::Cell(cell.clone())],
            domain.id().clone(),
        ),
    )
    .unwrap();
    drop(cell);
    assert!(weak.upgrade().is_some());
    collect(&domain);
    assert!(weak.upgrade().is_none());
}
#[test]
fn unreachable_cycles_do_not_change_shared_reachable_subgraphs_or_alias_identity() {
    let domain = domain();
    let shared = domain.new_cell(Value::Integer(41)).unwrap();
    let live = cycle(&domain);
    let dead = cycle(&domain);
    let weak = dead.downgrade();
    for cell in [&live, &dead] {
        let body = Value::Record(SharedRecord::new(
            Arc::new(RuntimeRecordLayout {
                record: RecordTypeId::new(0),
                type_name: "Shared".into(),
                fields: vec!["Self".into(), "Data".into()],
            }),
            vec![Value::Cell(cell.clone()), Value::Cell(shared.clone())],
        ));
        cell.replace(&domain, body).unwrap();
    }
    let root = domain.root_values(RootKind::Host, vec![Value::Cell(live.clone())]);
    drop(dead);
    collect(&domain);
    assert!(weak.upgrade().is_none());
    let Value::Record(body) = live.read(&domain).unwrap() else {
        panic!("record");
    };
    let Value::Cell(alias) = &body.body().values[1] else {
        panic!("shared cell");
    };
    assert!(alias.ptr_eq(&shared));
    assert_eq!(shared.read(&domain).unwrap(), Value::Integer(41));
    drop(body);
    drop(root);
    drop(live);
    drop(shared);
    collect(&domain);
    assert_eq!(domain.collection_stats().live_cells, 0);
}
#[test]
fn deep_shared_dags_and_wrappers_collect_on_a_small_native_stack() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let domain = domain();
            let cell = domain.new_cell(Value::Unit).unwrap();
            let weak = cell.downgrade();
            let mut value = Value::Cell(cell.clone());
            for _ in 0..20_000 {
                value = Value::Array(vec![value.clone(), value].into());
            }
            for _ in 0..60_000 {
                value = Value::option_some(value);
            }
            cell.replace(&domain, value).unwrap();
            let root = domain.root_values(RootKind::Host, vec![Value::Cell(cell)]);
            collect(&domain);
            assert!(weak.upgrade().is_some());
            drop(root);
            collect(&domain);
            assert!(weak.upgrade().is_none());
        })
        .unwrap()
        .join()
        .unwrap();
}
