//! Routine callback dependencies needed by initialization checks in consuming units.
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`.

/// Conservative invocation and returned-callable dependencies of one exported routine.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoutineInitialization {
    /// Canonical qualified routine identity, including record methods.
    pub qualified_name: String,
    /// Parameters whose callable values may be invoked, directly or transitively.
    pub called_parameters: Vec<usize>,
    /// Parameters whose callable values may be included in the returned value.
    pub returned_parameters: Vec<usize>,
    /// A call whose dependencies cannot be explained by parameter substitution.
    pub unknown_call: bool,
    /// Returned callable values not explained by returned parameters.
    pub unknown_return: bool,
}
