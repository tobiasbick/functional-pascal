//! Registered ownership of bounded live inspection copies and writable cell origins.

use super::render::RetainedValue;
use super::snapshot::InspectionSnapshot;
use super::targets::{MutationAccess, MutationPath, MutationRoot};
use fpas_bytecode::{MemoryDomain, RootKind, RootValues, Value};

pub(super) fn retain_values(values: &[RetainedValue]) -> Vec<RootValues> {
    let mut groups: Vec<(MemoryDomain, Vec<Value>)> = Vec::new();
    for value in values {
        let Some(domain) = &value.domain else {
            continue;
        };
        let index = groups
            .iter()
            .position(|(owner, _)| owner.id() == domain.id())
            .unwrap_or_else(|| {
                let index = groups.len();
                groups.push((domain.clone(), Vec::new()));
                index
            });
        let values = &mut groups[index].1;
        values.extend(value.value.iter().cloned());
        if let MutationAccess::Writable(target) = &value.origin {
            if let MutationRoot::ClosureCell(cell) = &target.root {
                values.push(Value::Cell(cell.clone()));
            }
            for step in &target.path {
                if let MutationPath::DictionaryValue(key) = step {
                    values.push(key.clone());
                }
            }
        }
    }
    groups
        .into_iter()
        .map(|(domain, values)| domain.root_values(RootKind::Inspection, values))
        .collect()
}

impl InspectionSnapshot {
    pub(super) fn register_captured_roots(&mut self) {
        self.roots = retain_values(&self.globals);
        for frame in &self.frames {
            self.roots.extend(retain_values(&frame.evaluation_values));
            self.roots.push(
                self.domain.root_values(
                    RootKind::Inspection,
                    frame
                        .bindings
                        .iter()
                        .filter_map(|binding| binding.value.clone())
                        .collect(),
                ),
            );
        }
    }

    pub(in crate::vm::debug) fn root_guards(&self) -> Vec<RootValues> {
        self.roots
            .iter()
            .chain(self.handles.iter().flat_map(|handle| &handle._roots))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::debug::evaluation::DebugEvaluationLimits;
    use crate::vm::debug::inspection::DebugInspectionLimits;
    use crate::vm::worker::Worker;
    use std::sync::Arc;

    #[test]
    fn an_actual_inspection_handle_keeps_a_detached_cycle_until_its_snapshot_expires() {
        use fpas_bytecode::MaintenanceOutcome;
        let (program, errors) = fpas_parser::parse("program Roots; begin end.");
        assert!(errors.is_empty());
        let worker = Worker::new(Arc::new(fpas_compiler::compile(&program).unwrap())).unwrap();
        let mut inspection =
            InspectionSnapshot::completed(&worker, 1, DebugInspectionLimits::default());
        let domain = crate::vm::domain::new_domain();
        let cell = domain.new_cell(Value::Unit).unwrap();
        let weak = cell.downgrade();
        cell.replace(&domain, Value::option_some(Value::Cell(cell.clone())))
            .unwrap();
        let result = inspection
            .retain_evaluation_result_in_domain(
                Value::Array(vec![Value::Cell(cell)].into()),
                domain.clone(),
                DebugEvaluationLimits::default(),
            )
            .unwrap();
        drop(worker);
        assert!(matches!(
            domain.request_explicit_maintenance(),
            MaintenanceOutcome::Serviced(_)
        ));
        assert!(weak.upgrade().is_some());
        assert!(
            !inspection
                .variables(result.variables_reference, 0, 1)
                .unwrap()
                .items
                .is_empty()
        );
        drop(inspection);
        assert!(matches!(
            domain.request_explicit_maintenance(),
            MaintenanceOutcome::Serviced(_)
        ));
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn detached_children_keep_their_domain_until_the_inspection_handle_expires() {
        let (program, errors) = fpas_parser::parse("program Roots; begin end.");
        assert!(errors.is_empty());
        let worker = Worker::new(Arc::new(fpas_compiler::compile(&program).expect("image")))
            .expect("worker");
        let mut inspection =
            InspectionSnapshot::completed(&worker, 1, DebugInspectionLimits::default());
        let domain = MemoryDomain::new();
        let cell = domain.new_cell(Value::Integer(42)).expect("detached cell");
        let weak = cell.downgrade();
        let value = Value::Array(vec![Value::Cell(cell)].into());
        let temporary = domain.root_values(RootKind::Evaluation, vec![value.clone()]);
        let result = inspection
            .retain_evaluation_result_in_domain(
                value,
                domain.clone(),
                DebugEvaluationLimits::default(),
            )
            .expect("retain");
        drop(temporary);
        assert_eq!(domain.root_count(), 1);
        let cell_view = inspection
            .variables(result.variables_reference, 0, 1)
            .expect("cell view")
            .items
            .remove(0);
        assert_eq!(cell_view.value, "<mutable cell>");
        assert_eq!(
            inspection
                .variables(cell_view.variables_reference, 0, 1)
                .expect("payload")
                .items[0]
                .value,
            "42"
        );
        assert!(weak.upgrade().is_some());
        drop(inspection);
        assert_eq!(domain.root_count(), 0);
        assert!(weak.upgrade().is_none());
    }
}
