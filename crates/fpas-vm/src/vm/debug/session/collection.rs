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
    fn continued_execution_collects_cycles_and_keeps_live_captures_across_stops() {
        use crate::vm::debug::{DebugRunResult, DebugStopReason, FunctionBreakpoint};
        use std::num::NonZeroUsize;
        let (program, diagnostics) = fpas_parser::parse(
            r#"program DebugCycles;
uses Std.Console as Console;
function Make(): function(): integer;
begin
  var Count := 0;
  var Current: function(): integer := function(): integer begin return 0; end function;
  Current := function(): integer begin
    Count := Count + 1;
    if Count < 0 then return Current(); end if;
    return Count;
  end function;
  return Current;
end function;
procedure Marker(); begin Console.WriteLn('marker'); end procedure;
begin
  const Keep := Make();
  const Alias := Keep;
  for I: integer := 1 to 300 do
    const Garbage := Make();
    if Garbage() <> 1 then panic('garbage state'); end if;
    if Keep() <> I * 2 - 1 then panic('live state'); end if;
    if Alias() <> I * 2 then panic('alias state'); end if;
  end for;
  Marker();
  if Keep() <> 601 then panic('capture lost at stop'); end if;
end."#,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let config = CollectionSchedule {
            allocations: NonZeroUsize::new(8).unwrap(),
            mutations: NonZeroUsize::new(16).unwrap(),
            safe_points: NonZeroUsize::new(4).unwrap(),
        };
        let mut session = DebugSession::new(fpas_compiler::compile(&program).unwrap())
            .unwrap()
            .with_collection_schedule(config);
        let domain = session.runtime.worker(0).unwrap().domain.clone();
        let breakpoints = session
            .replace_function_breakpoints(vec![FunctionBreakpoint {
                name: "Marker".into(),
            }])
            .unwrap();
        assert!(breakpoints[0].is_verified());
        let DebugRunResult::Stopped(stop) = session.continue_execution().unwrap() else {
            panic!("expected marker breakpoint");
        };
        assert_eq!(stop.reason, DebugStopReason::Breakpoint);
        assert!(domain.collection_stats().cleared_cells > 100);
        // A stop must release exclusion so a different thread can service roots.
        let (sent, received) = std::sync::mpsc::channel();
        let collector = std::thread::spawn(move || {
            sent.send(domain.request_explicit_maintenance()).unwrap();
        });
        assert!(matches!(
            received
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap(),
            fpas_bytecode::MaintenanceOutcome::Serviced(_)
        ));
        collector.join().unwrap();
        assert!(matches!(
            session.continue_execution().unwrap(),
            DebugRunResult::Terminated(_)
        ));
    }
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
