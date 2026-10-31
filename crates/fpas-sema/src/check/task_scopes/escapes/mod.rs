//! Task handles of scope-owned children must not outlive their scope (Q27).
//!
//! The analysis follows values whose type can hold a task handle from `go` expressions and
//! routine parameters through bindings, aggregates, and calls. Scope-owned handles that reach
//! a return, outer storage, a channel send, a capture, or an escaping parameter are rejected.
//! Parameter escapes of all routines are inferred together and exported in unit interfaces.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`.

mod calls;
mod captures;
mod expressions;
mod loops;
mod origins;
mod routines;
mod statements;
mod walker;

use std::collections::HashMap;

use fpas_diagnostics::codes::SEMA_TASK_HANDLE_ESCAPE;
use fpas_parser::{Decl, Stmt};

use super::super::Checker;
use crate::scope::canonical_symbol_name;
use routines::RoutineTable;
use walker::{EscapeError, Walker};

impl Checker {
    /// Infer parameter escapes and reject scope-owned task handles that outlive their scope.
    pub(in crate::check) fn check_task_handle_escapes(
        &mut self,
        declarations: &[Decl],
        body: Option<&[Stmt]>,
    ) {
        let table = RoutineTable::collect(declarations, self);
        let mut flags: Vec<Vec<bool>> = table
            .routines
            .iter()
            .map(|routine| vec![false; routine.params.len() + routine.captures.len()])
            .collect();
        // Escapes only grow, so the iteration reaches the least fixpoint.
        loop {
            let mut changed = false;
            for index in 0..table.routines.len() {
                let (escapes, _) = self.walk_routine(&table, &flags, index);
                for (flag, escapes) in flags[index].iter_mut().zip(escapes) {
                    if escapes && !*flag {
                        *flag = true;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        let mut errors = Vec::new();
        for index in 0..table.routines.len() {
            errors.extend(self.walk_routine(&table, &flags, index).1);
        }
        {
            let mut walker = Walker::new(self, &table, &flags, &[]);
            walker.walk_initializers(declarations);
            if let Some(body) = body {
                walker.walk_statements(body);
            }
            errors.extend(walker.errors);
        }
        for error in errors {
            self.report_task_handle_escape(error);
        }
        self.routine_param_escapes = table
            .routines
            .iter()
            .zip(flags)
            .map(|(routine, mut flags)| {
                flags.truncate(routine.params.len());
                (canonical_symbol_name(&routine.name), flags)
            })
            .collect::<HashMap<_, _>>();
    }

    fn walk_routine(
        &self,
        table: &RoutineTable<'_>,
        flags: &[Vec<bool>],
        index: usize,
    ) -> (Vec<bool>, Vec<EscapeError>) {
        let routine = &table.routines[index];
        let mut walker = Walker::new(self, table, flags, routine.params);
        walker.install_routine_captures(index);
        walker.walk_body(routine.body, Some(index));
        (walker.escapes, walker.errors)
    }

    fn report_task_handle_escape(&mut self, error: EscapeError) {
        if error.resource {
            self.error_with_code(SEMA_TASK_HANDLE_ESCAPE,
                format!("A resource owned by the `using` binding at line {} would escape through {}", error.scope.line, error.route),
                "Keep the resource within its owning scope and pass it only to routines that do not retain it.", error.span);
            return;
        }
        self.error_with_code(
            SEMA_TASK_HANDLE_ESCAPE,
            format!(
                "A task handle owned by the `scope` at line {} would escape it through {}",
                error.scope.line, error.route
            ),
            "A scope waits for or cancels its children when it ends, so their handles must stay inside it. Keep handles in bindings declared inside the scope, wait for them there, and pass them only to routines that do not store, return, send, or capture them.",
            error.span,
        );
    }
}
