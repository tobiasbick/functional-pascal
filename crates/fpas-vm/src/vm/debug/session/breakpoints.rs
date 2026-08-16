//! Atomic session ownership for source and function breakpoints.

use super::*;
use crate::vm::debug::breakpoints as binding;

impl DebugSession {
    /// Return the breakpoint resource bounds enforced by this session.
    #[must_use]
    pub const fn breakpoint_limits(&self) -> DebugBreakpointLimits {
        self.breakpoint_limits
    }

    /// Add one source breakpoint and return its verified or unverified binding.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or breakpoint-limit error.
    pub fn set_breakpoint(
        &mut self,
        requested: SourceBreakpoint,
    ) -> Result<BoundBreakpoint, DebugSessionError> {
        self.require_stopped("breakpoint.set")?;
        self.require_breakpoint_capacity(
            self.source_breakpoints.len() + self.function_breakpoints.len() + 1,
        )?;
        let id = self.take_breakpoint_ids(1)?;
        let breakpoint = binding::bind_source(&self.executable, id, requested);
        self.source_breakpoints.push(breakpoint.clone());
        Ok(breakpoint)
    }

    /// Atomically replace every function breakpoint in the session.
    ///
    /// Each selector binds all matching exact function identities in executable
    /// order. Selectors without an executable entry sequence point are retained
    /// as unverified logical breakpoints.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or breakpoint-limit error without changing the
    /// existing function breakpoints or the next logical identifier.
    pub fn replace_function_breakpoints(
        &mut self,
        requested: Vec<FunctionBreakpoint>,
    ) -> Result<Vec<BoundFunctionBreakpoint>, DebugSessionError> {
        self.require_stopped("function_breakpoints.replace")?;
        self.require_breakpoint_capacity(self.source_breakpoints.len() + requested.len())?;
        let count = u64::try_from(requested.len()).map_err(|_| breakpoint_id_limit())?;
        let next_id = self
            .next_breakpoint_id
            .checked_add(count)
            .ok_or_else(breakpoint_id_limit)?;
        let mut bound = Vec::with_capacity(requested.len());
        for (offset, request) in requested.into_iter().enumerate() {
            if request.name.is_empty()
                || request.name.len() > self.breakpoint_limits.max_function_name_bytes
            {
                return Err(DebugSessionError {
                    kind: DebugErrorKind::BreakpointLimit,
                    message: format!(
                        "function breakpoint selector must contain 1..={} UTF-8 bytes",
                        self.breakpoint_limits.max_function_name_bytes
                    ),
                    hint: "Use a non-empty canonical or short function name.".to_string(),
                });
            }
            let offset = u64::try_from(offset).map_err(|_| breakpoint_id_limit())?;
            let breakpoint =
                binding::bind_function(&self.executable, self.next_breakpoint_id + offset, request);
            if breakpoint.functions.len() > self.breakpoint_limits.max_function_bindings {
                return Err(DebugSessionError {
                    kind: DebugErrorKind::BreakpointLimit,
                    message: format!(
                        "function breakpoint `{}` matches {} functions; the session limit is {}",
                        breakpoint.requested.name,
                        breakpoint.functions.len(),
                        self.breakpoint_limits.max_function_bindings
                    ),
                    hint: "Use a more qualified function name to narrow the exact matches."
                        .to_string(),
                });
            }
            bound.push(breakpoint);
        }
        self.function_breakpoints.clone_from(&bound);
        self.next_breakpoint_id = next_id;
        Ok(bound)
    }

    /// Remove one source or function breakpoint by its logical session ID.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or unknown-breakpoint error.
    pub fn clear_breakpoint(&mut self, id: u64) -> Result<(), DebugSessionError> {
        self.require_stopped("breakpoint.clear")?;
        if let Some(index) = self
            .source_breakpoints
            .iter()
            .position(|breakpoint| breakpoint.id == id)
        {
            self.source_breakpoints.remove(index);
            return Ok(());
        }
        if let Some(index) = self
            .function_breakpoints
            .iter()
            .position(|breakpoint| breakpoint.id == id)
        {
            self.function_breakpoints.remove(index);
            return Ok(());
        }
        Err(DebugSessionError {
            kind: DebugErrorKind::UnknownBreakpoint,
            message: format!("debug breakpoint {id} does not exist"),
            hint: "Use an ID returned by a breakpoint command in this session.".to_string(),
        })
    }

    fn require_breakpoint_capacity(&self, requested_total: usize) -> Result<(), DebugSessionError> {
        if requested_total <= self.breakpoint_limits.max_breakpoints {
            return Ok(());
        }
        Err(DebugSessionError {
            kind: DebugErrorKind::BreakpointLimit,
            message: format!(
                "debug session would retain {requested_total} logical breakpoints; the limit is {}",
                self.breakpoint_limits.max_breakpoints
            ),
            hint: "Clear breakpoints or send a smaller replace request.".to_string(),
        })
    }

    fn take_breakpoint_ids(&mut self, count: u64) -> Result<u64, DebugSessionError> {
        let first = self.next_breakpoint_id;
        self.next_breakpoint_id = first.checked_add(count).ok_or_else(breakpoint_id_limit)?;
        Ok(first)
    }
}

fn breakpoint_id_limit() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::BreakpointLimit,
        message: "debug session exhausted its logical breakpoint identifiers".to_string(),
        hint: "Start a new debug session before setting more breakpoints.".to_string(),
    }
}
