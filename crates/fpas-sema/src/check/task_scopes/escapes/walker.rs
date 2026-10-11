//! Shared state of one routine-body walk: lexical frames, open scopes, and escapes found.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`.

use fpas_lexer::Span;
use fpas_parser::{Expr, FormalParam, ParamMode};

use super::origins::{Binding, Frame, Origin, Owners};
use super::routines::RoutineTable;
use crate::check::Checker;
use crate::scope::canonical_symbol_name;

/// A scope-owned handle that would outlive its scope.
pub(super) struct EscapeError {
    /// Expression or statement through which the handle leaves.
    pub(super) span: Span,
    /// The owning `scope` statement.
    pub(super) scope: Span,
    /// How the handle leaves, phrased to complete "would escape its scope through ...".
    pub(super) route: String,
}

/// Result of resolving a name at the current lexical position.
pub(super) enum Resolved {
    /// Local binding of the current routine body; the index addresses `Walker::frames`.
    Local(usize),
    /// Storage outside the current routine body.
    Outer,
    /// Named routine of the analyzed unit.
    Routine(usize),
    /// Unit-level value, imported symbol, or unknown name.
    Global,
}

/// Walks one routine body and records where tracked values may escape.
pub(super) struct Walker<'c, 'a> {
    pub(super) checker: &'c Checker,
    pub(super) table: &'c RoutineTable<'a>,
    pub(super) flags: &'c [Vec<bool>],
    pub(super) frames: Vec<Frame>,
    /// Spans of the open `scope` statements of the current routine body, outermost first.
    pub(super) scopes: Vec<Span>,
    /// Parameters of the analyzed routine whose values may escape.
    pub(super) escapes: Vec<bool>,
    pub(super) errors: Vec<EscapeError>,
}

impl<'c, 'a> Walker<'c, 'a> {
    /// Start a walk with the analyzed routine's parameters as tracked sources.
    pub(super) fn new(
        checker: &'c Checker,
        table: &'c RoutineTable<'a>,
        flags: &'c [Vec<bool>],
        params: &[FormalParam],
    ) -> Self {
        let mut frame = Frame::default();
        for (index, param) in params.iter().enumerate() {
            frame.bindings.insert(
                canonical_symbol_name(&param.name),
                Binding {
                    owners: vec![Origin::Parameter(index)],
                    depth: 0,
                    outer: param.mode == ParamMode::Var,
                },
            );
        }
        Self {
            checker,
            table,
            flags,
            frames: vec![frame],
            scopes: Vec::new(),
            escapes: vec![false; params.len()],
            errors: Vec::new(),
        }
    }

    /// Current task-scope depth of the routine body.
    pub(super) fn depth(&self) -> u32 {
        self.scopes.len() as u32
    }

    /// Declare a local binding at the current position.
    pub(super) fn declare(&mut self, name: &str, owners: Owners) {
        let depth = self.depth();
        if let Some(frame) = self.frames.last_mut() {
            frame.bindings.insert(
                canonical_symbol_name(name),
                Binding {
                    owners,
                    depth,
                    outer: false,
                },
            );
        }
    }

    /// Resolve a name through local frames, then the unit's routines.
    pub(super) fn resolve(&self, name: &str) -> Resolved {
        let canonical = canonical_symbol_name(name);
        let mut crossed_boundary = false;
        for (index, frame) in self.frames.iter().enumerate().rev() {
            if frame.bindings.contains_key(&canonical) {
                return if crossed_boundary {
                    Resolved::Outer
                } else {
                    Resolved::Local(index)
                };
            }
            if let Some(routine) = frame.routines.get(&canonical) {
                return Resolved::Routine(*routine);
            }
            crossed_boundary |= frame.routine_boundary;
        }
        self.table
            .top_level
            .get(&canonical)
            .map_or(Resolved::Global, |routine| Resolved::Routine(*routine))
    }

    /// Origins a read of `name` yields.
    pub(super) fn read(&self, name: &str) -> Owners {
        match self.resolve(name) {
            Resolved::Local(frame) => self.frames[frame].bindings[&canonical_symbol_name(name)]
                .owners
                .clone(),
            Resolved::Routine(index) => self.routine_capture_origins(index),
            Resolved::Outer | Resolved::Global => Vec::new(),
        }
    }

    /// Store `owners` into the binding `name`, reporting every origin that would escape.
    pub(super) fn store(&mut self, name: &str, owners: Owners, span: Span) {
        if owners.is_empty() {
            return;
        }
        let Resolved::Local(frame) = self.resolve(name) else {
            self.escape_all(
                &owners,
                span,
                &format!("storage in `{name}` outside the scope"),
            );
            return;
        };
        let canonical = canonical_symbol_name(name);
        let mut escaped = Vec::new();
        let Some(binding) = self.frames[frame].bindings.get_mut(&canonical) else {
            // Missing storage cannot prove that the value remains local.
            self.escape_all(
                &owners,
                span,
                &format!("storage in `{name}` outside the scope"),
            );
            return;
        };
        for origin in owners {
            if binding.may_hold(origin) {
                if !binding.owners.contains(&origin) {
                    binding.owners.push(origin);
                }
            } else {
                escaped.push(origin);
            }
        }
        self.escape_all(
            &escaped,
            span,
            &format!("storage in `{name}`, which is declared outside the scope"),
        );
    }

    /// Report or record every origin as leaving its region.
    pub(super) fn escape_all(&mut self, owners: &[Origin], span: Span, route: &str) {
        for origin in owners {
            match *origin {
                Origin::Scope { span: scope, .. } => {
                    if !self
                        .errors
                        .iter()
                        .any(|error| error.span == span && error.scope == scope)
                    {
                        self.errors.push(EscapeError {
                            span,
                            scope,
                            route: route.to_string(),
                        });
                    }
                }
                Origin::Parameter(index) => {
                    if let Some(escapes) = self.escapes.get_mut(index) {
                        *escapes = true;
                    }
                }
            }
        }
    }

    /// Keep origins only for values whose type can hold a task handle.
    pub(super) fn filtered(&self, expr: &Expr, owners: Owners) -> Owners {
        if owners.is_empty() {
            return owners;
        }
        let holds = self
            .checker
            .expr_types
            .get(&Checker::expr_lookup_key(expr))
            .is_none_or(|ty| self.checker.may_hold_task_handle(ty));
        if holds { owners } else { Vec::new() }
    }

    /// Run `walk` in a new lexical frame.
    pub(super) fn in_frame(&mut self, walk: impl FnOnce(&mut Self)) {
        self.frames.push(Frame::default());
        walk(self);
        self.frames.pop();
    }
}
