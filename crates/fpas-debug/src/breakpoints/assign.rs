//! Optional assignment attached to a logical breakpoint policy.

use fpas_vm::{DebugDataLocationIdentity, DebugExpression};

/// One global assignment executed after a breakpoint's condition and hit test.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BreakpointAssign {
    pub(crate) identity: DebugDataLocationIdentity,
    pub(crate) expression: DebugExpression,
}

impl BreakpointAssign {
    pub(crate) fn new(
        identity: DebugDataLocationIdentity,
        expression: DebugExpression,
    ) -> Result<Self, String> {
        match identity {
            DebugDataLocationIdentity::Global { .. } => Ok(Self {
                identity,
                expression,
            }),
            DebugDataLocationIdentity::FrameRegister { .. } => Err(
                "Breakpoint assign requires a global identity from `location.describe`."
                    .to_string(),
            ),
        }
    }
}
