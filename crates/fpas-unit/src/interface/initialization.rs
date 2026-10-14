//! Routine callback dependencies needed by initialization checks in consuming units.
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`.

/// Conservative invocation and returned-callable dependencies of one exported routine.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoutineInitialization {
    /// Canonical qualified routine identity, including record methods.
    pub qualified_name: String,
    /// Parameters whose callable values may be invoked, directly or transitively.
    pub called_parameters: Vec<usize>,
    /// Callable parameters executed across a worker-task boundary.
    pub started_parameters: Vec<usize>,
    /// Mutable program/unit variables accessed directly or through callees.
    pub mutable_globals: Vec<String>,
    /// Canonical unit variables written directly or through callees, used by for-in protection.
    /// **Documentation:** `docs/pascal/language/control-flow/for-in.md`.
    pub written_globals: Vec<String>,
    /// Worker dependencies that cannot be explained by parameter substitution.
    pub unknown_worker: bool,
    /// Parameters whose callable values may be included in the returned value.
    pub returned_parameters: Vec<usize>,
    /// A call whose dependencies cannot be explained by parameter substitution.
    pub unknown_call: bool,
    /// Returned callable values not explained by returned parameters.
    pub unknown_return: bool,
}
