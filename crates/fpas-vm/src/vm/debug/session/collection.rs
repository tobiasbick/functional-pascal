//! Collection configuration of stopped sessions and their helper domains.
use super::DebugSession;

impl DebugSession {
    /// Effective collection thresholds inherited by tasks and debugger helpers.
    #[must_use]
    pub fn collection_schedule(&self) -> fpas_bytecode::CollectionSchedule {
        self.runtime
            .worker(0)
            .and_then(|worker| worker.domain.collection_schedule())
            .unwrap_or_default()
    }

    /// Select collection thresholds before launching this stopped debug session.
    #[must_use]
    pub fn with_collection_schedule(mut self, schedule: fpas_bytecode::CollectionSchedule) -> Self {
        self.runtime.configure_collection(schedule);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::debug::calls::ValueDetacher;
    use fpas_bytecode::{CollectionSchedule, RootKind, Value};
    #[test]
    fn overrides_reach_detached_helpers_and_inspection_churn_releases_cycles() {
        let (program, errors) = fpas_parser::parse("program DebugCycles; begin end.");
        assert!(errors.is_empty());
        let config = CollectionSchedule {
            allocations: std::num::NonZeroUsize::new(3).unwrap(),
            mutations: std::num::NonZeroUsize::new(7).unwrap(),
            safe_points: std::num::NonZeroUsize::new(11).unwrap(),
        };
        let session = DebugSession::new(fpas_compiler::compile(&program).unwrap())
            .unwrap()
            .with_collection_schedule(config);
        assert_eq!(session.collection_schedule(), config);
        let source = session.runtime.worker(0).unwrap().domain.clone();
        for _ in 0..40 {
            let access = source.access();
            let cell = source.new_cell(Value::Unit).unwrap();
            cell.replace(&source, Value::option_some(Value::Cell(cell.clone())))
                .unwrap();
            let roots = source.root_values(RootKind::Host, vec![Value::Cell(cell.clone())]);
            drop(access);
            let mut detacher = ValueDetacher::new(100);
            detacher.allow_source(source.clone());
            let value = detacher.observation(&Value::Cell(cell)).unwrap();
            let helper = detacher.domain.clone();
            assert_eq!(helper.collection_schedule(), Some(config));
            let retained = helper.root_values(RootKind::Inspection, vec![value]);
            drop(detacher);
            assert_eq!(helper.collection_stats().live_cells, 1);
            drop(retained);
            assert_eq!(helper.collection_stats().live_cells, 0);
            drop(roots);
            assert_eq!(source.collection_stats().live_cells, 0);
        }
    }
}
