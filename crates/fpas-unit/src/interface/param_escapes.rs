//! Parameters of exported routines that may outlive a call.

/// Inferred parameter escapes of one exported routine or record routine.
///
/// A scope-owned task handle may only be passed to a parameter that does not escape.
/// **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoutineParamEscapes {
    /// Fully qualified definition name, including the record owner for record routines.
    pub qualified_name: String,
    /// One entry per declared parameter, including `Self`; empty when none escapes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub escapes: Vec<bool>,
}
