//! Canonical parameter modes shared by executable and unit-object validation.

/// Return whether var positions are ordered, distinct and inside the argument window.
#[must_use]
pub fn var_parameters_are_valid(arity: u8, positions: &[u8]) -> bool {
    positions.last().is_none_or(|position| *position < arity)
        && positions.windows(2).all(|pair| pair[0] < pair[1])
}
