//! Intrinsics for registered MCP tools and cooperative FPAS serving.

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Operations for the independent Std.Ai.Mcp library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum McpIntrinsic {
    DefaultLimits = 804,
    DefaultHttpOptions = 805,
    Create = 806,
    RegisterTool = 807,
    Publish = 808,
    Access = 809,
    CloseEndpoint = 810,
    Close = 811,
    Release = 812,
    ParseJson = 813,
    JsonMember = 814,
    ParseToolResult = 815,
    TextResult = 816,
    ErrorResult = 817,
    StructuredResult = 818,
    IsCancelled = 819,
    ServeOne = 820,
    Serve = 821,
}
}

impl McpIntrinsic {
    /// Operations whose lifetime is polled by the owning FPAS task.
    pub const fn waits(self) -> bool {
        matches!(
            self,
            Self::Publish
                | Self::CloseEndpoint
                | Self::Close
                | Self::Release
                | Self::ServeOne
                | Self::Serve
        )
    }
    /// Canonical source declaration of this operation.
    pub fn source_name(self) -> String {
        format!("Std.Ai.Mcp.{self:?}")
    }
}
