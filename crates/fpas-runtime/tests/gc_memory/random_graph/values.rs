//! Runtime encodings; the reference model stores only integer adjacency lists.

use fpas_runtime::{
    EnumTypeId, EnumVariantId, FunctionTarget, MemoryDomain, OwnedCell, RecordTypeId,
    ReferenceRoot, RuntimeEnumLayout, RuntimeRecordLayout, SharedEnum, SharedRecord, Value,
    VariableReference, WeakOwnedCell,
};
use std::sync::Arc;

pub(super) struct Shapes {
    pub target: FunctionTarget,
    record: Arc<RuntimeRecordLayout>,
    enumeration: Arc<RuntimeEnumLayout>,
}

impl Shapes {
    pub fn new(target: FunctionTarget) -> Self {
        Self {
            target,
            record: Arc::new(RuntimeRecordLayout {
                record: RecordTypeId::new(0),
                type_name: "GraphLink".into(),
                fields: vec!["Links".into()],
            }),
            enumeration: Arc::new(RuntimeEnumLayout {
                error: None,
                enumeration: EnumTypeId::new(0),
                variant_id: EnumVariantId::new(0),
                type_name: "GraphLink".into(),
                variant: "Links".into(),
                fields: vec!["Links".into()],
            }),
        }
    }

    fn wrap(&self, kind: u8, value: Value) -> Value {
        match kind {
            0 => Value::Array(vec![value].into()),
            1 => Value::dict(vec![(Value::Integer(0), value)]),
            2 => Value::dict(vec![(value, Value::Integer(0))]),
            3 => Value::Record(SharedRecord::new(self.record.clone(), vec![value])),
            4 => Value::Enum(SharedEnum::new(self.enumeration.clone(), vec![value])),
            5 => Value::option_some(value),
            6 => Value::result_ok(value),
            7 => Value::result_error(value),
            8 => Value::function(self.target.clone(), vec![value]),
            9 => Value::bound_function(self.target.clone(), value),
            _ => unreachable!(),
        }
    }

    pub fn payload(&self, id: usize, node: &super::Node, nodes: &[super::Node]) -> Value {
        let mut edges = Value::Array(
            node.edges
                .iter()
                .map(|id| Value::Cell(cell(&nodes[*id].weak)))
                .collect::<Vec<_>>()
                .into(),
        );
        for kind in &node.layers {
            edges = self.wrap(*kind, edges);
        }
        let marker = (id + node.revision) as i64;
        let array = if id.is_multiple_of(12) {
            Value::Array(vec![Value::Integer(marker); 8192].into())
        } else {
            Value::Unit
        };
        Value::Array(
            vec![
                Value::Integer(id as i64),
                Value::Integer(node.revision as i64),
                Value::Str(
                    char::from(b'a' + (marker % 26) as u8)
                        .to_string()
                        .repeat(64 * 1024)
                        .into(),
                ),
                edges,
                array,
            ]
            .into(),
        )
    }

    pub fn root(&self, id: usize, nodes: &[super::Node], reference: bool) -> Value {
        let cell = cell(&nodes[id].weak);
        if reference {
            Value::Reference(Arc::new(VariableReference {
                root: ReferenceRoot::Cell(cell),
                path: vec![fpas_runtime::ReferenceStep::Element(0)],
            }))
        } else {
            Value::option_some(Value::function(
                self.target.clone(),
                vec![Value::Cell(cell)],
            ))
        }
    }
}

pub(super) fn cell(weak: &WeakOwnedCell) -> OwnedCell {
    weak.upgrade()
        .expect("model-selected cell must still exist")
}

pub(super) fn check_root(value: &Value, id: usize, nodes: &[super::Node]) {
    let actual = match value {
        Value::Reference(reference) => {
            assert!(matches!(
                reference.path.as_slice(),
                [fpas_runtime::ReferenceStep::Element(0)]
            ));
            let ReferenceRoot::Cell(cell) = &reference.root else {
                panic!("wrong reference root")
            };
            cell
        }
        Value::OptionSome(function) => {
            let Value::Function(function) = &**function else {
                panic!("wrong root wrapper")
            };
            let Value::Cell(cell) = &function.captures[0] else {
                panic!("wrong root capture")
            };
            cell
        }
        _ => panic!("wrong root value"),
    };
    assert!(actual.ptr_eq(&cell(&nodes[id].weak)), "root target {id}");
}

pub(super) fn check(id: usize, node: &super::Node, nodes: &[super::Node], domain: &MemoryDomain) {
    let live = cell(&node.weak);
    let value = live.read_snapshot(domain).unwrap();
    let Value::Array(fields) = &*value else {
        panic!("live payload cleared: node {id}")
    };
    assert_eq!(fields[0], Value::Integer(id as i64));
    assert_eq!(fields[1], Value::Integer(node.revision as i64));
    let marker = (id + node.revision) as i64;
    let Value::Str(text) = &fields[2] else {
        panic!("missing string")
    };
    assert_eq!(text.len(), 64 * 1024);
    assert_eq!(text.as_bytes()[0], b'a' + (marker % 26) as u8);
    assert_eq!(text.as_bytes()[text.len() - 1], b'a' + (marker % 26) as u8);
    if id.is_multiple_of(12) {
        let Value::Array(array) = &fields[4] else {
            panic!("missing large array")
        };
        assert_eq!(array.len(), 8192);
        assert_eq!(array[0], Value::Integer(marker));
        assert_eq!(array[8191], Value::Integer(marker));
    }
    let mut edges = &fields[3];
    for kind in node.layers.iter().rev() {
        edges = match (kind, edges) {
            (0, Value::Array(array)) => &array[0],
            (1, Value::Dict(pairs)) => &pairs[0].1,
            (2, Value::Dict(pairs)) => &pairs[0].0,
            (3, Value::Record(record)) => &record.body().values[0],
            (4, Value::Enum(enumeration)) => &enumeration.body().values[0],
            (5, Value::OptionSome(inner))
            | (6, Value::ResultOk(inner))
            | (7, Value::ResultError(inner)) => inner,
            (8, Value::Function(function)) => &function.captures[0],
            (9, Value::Function(function)) => function.bound_receiver.as_ref().unwrap(),
            _ => panic!("wrong wrapper {kind}: node {id}"),
        };
    }
    let Value::Array(edges) = edges else {
        panic!("missing edges")
    };
    assert_eq!(edges.len(), node.edges.len());
    for (edge, expected) in edges.iter().zip(&node.edges) {
        let Value::Cell(actual) = edge else {
            panic!("missing cell edge")
        };
        assert!(
            actual.ptr_eq(&cell(&nodes[*expected].weak)),
            "node {id}, target {expected}"
        );
    }
}
