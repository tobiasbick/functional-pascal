//! Static evidence for transferring a value graph across task boundaries.

/// Known ownership restrictions of a value, independently of its callable signature.
///
/// Unknown values remain subject to runtime transfer checks. A task-bound alternative
/// remains visible when branch or aggregate states are joined.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransferCapability {
    /// The analyzed value contains no task-local captures.
    Transferable,
    /// Available metadata does not prove whether the value contains local captures.
    #[default]
    Unknown,
    /// At least one known constituent or execution path retains task-local captures.
    TaskBound,
}

impl TransferCapability {
    /// Combine constituents or alternatives without erasing known local ownership.
    #[must_use]
    pub const fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::TaskBound, _) | (_, Self::TaskBound) => Self::TaskBound,
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::Transferable, Self::Transferable) => Self::Transferable,
        }
    }

    /// Whether static evidence is sufficient to reject a transfer.
    #[must_use]
    pub const fn is_task_bound(self) -> bool {
        matches!(self, Self::TaskBound)
    }
}

#[cfg(test)]
mod tests {
    use super::TransferCapability::{TaskBound, Transferable, Unknown};

    #[test]
    fn joins_retain_local_evidence_and_unknown_inputs() {
        let states = [Transferable, Unknown, TaskBound];
        for (left_index, left) in states.into_iter().enumerate() {
            for (right_index, right) in states.into_iter().enumerate() {
                assert_eq!(left.join(right), states[left_index.max(right_index)]);
                assert_eq!(left.join(right), right.join(left));
                for third in states {
                    assert_eq!(left.join(right).join(third), left.join(right.join(third)));
                }
            }
            assert_eq!(left.join(left), left);
        }
        assert_eq!(super::TransferCapability::default(), Unknown);
        assert!(!Unknown.is_task_bound());
    }
}
