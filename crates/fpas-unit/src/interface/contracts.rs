//! Public routine contracts retained independently of callable type identity.
//! **Documentation:** `docs/pascal/language/functions/contracts.md`.

/// Entry and return clauses belonging to one exported routine or record method.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoutineContract {
    /// Fully qualified definition name, including the record owner for methods.
    pub qualified_name: String,
    /// Original precondition clauses in evaluation order.
    pub requires: Vec<String>,
    /// Original postcondition clauses, including result names, in evaluation order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ensures: Vec<String>,
}
