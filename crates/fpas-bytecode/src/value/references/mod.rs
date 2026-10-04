//! Storage-root access shared by capture cells and synchronous caller references.

mod borrow;
mod path;
mod registry;
mod selection;

#[cfg(test)]
mod tests;

pub use borrow::CellBorrow;
pub use path::{ReferencePathError, ReferenceStep};
pub use registry::ReferenceRegistry;
pub use selection::SelectedReference;

/// A storage access or reference transition lacks the required authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceError {
    /// Argument evaluation has reserved the root against mutation or a second borrow.
    Reserved,
    /// An active reference excludes access through an unrelated alias.
    Exclusive,
    /// A child reborrow has temporarily suspended its parent's authority.
    Suspended,
    /// A released reference cannot access storage or enter another call.
    Released,
    /// A reference must enter the callee before it can be forwarded.
    NotActive,
    /// An already active reference cannot enter a second call without reborrowing.
    AlreadyActive,
    /// The registry exhausted its non-reused reference identities.
    IdentityLimit,
}

impl std::fmt::Display for ReferenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Reserved => "Storage root is reserved while evaluating var arguments",
            Self::Exclusive => "Storage root is exclusively accessed through a var reference",
            Self::Suspended => "Parent var reference is suspended by a child reborrow",
            Self::Released => "Var reference has already been released",
            Self::NotActive => "Only an active var parameter can be forwarded",
            Self::AlreadyActive => {
                "Var reference is already active; forwarding requires a reborrow"
            }
            Self::IdentityLimit => "Runtime var reference identity limit exceeded",
        })
    }
}

impl std::error::Error for ReferenceError {}
