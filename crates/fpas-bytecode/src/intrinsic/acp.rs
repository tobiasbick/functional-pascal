//! Stable selectors for detached ACP values.

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Acp.Types`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum AcpIntrinsic {
    ParseRawJson = 735,
    RawJsonText = 736,
    ParseUInt64 = 737,
    UInt64Text = 738,
    UInt64FromInteger = 739,
    UInt64ToInteger = 740,
    DecodeClientRequest = 741,
    EncodeClientRequest = 742,
    DecodeAgentResponse = 743,
    EncodeAgentResponse = 744,
    DecodeClientNotification = 745,
    EncodeClientNotification = 746,
    DecodeAgentRequest = 747,
    EncodeAgentRequest = 748,
    DecodeClientResponse = 749,
    EncodeClientResponse = 750,
    DecodeAgentNotification = 751,
    EncodeAgentNotification = 752,
    DecodeProtocolLevelNotification = 753,
    EncodeProtocolLevelNotification = 754,
    V2DecodeClientRequest = 755,
    V2EncodeClientRequest = 756,
    V2DecodeAgentResponse = 757,
    V2EncodeAgentResponse = 758,
    V2DecodeClientNotification = 759,
    V2EncodeClientNotification = 760,
    V2DecodeAgentRequest = 761,
    V2EncodeAgentRequest = 762,
    V2DecodeClientResponse = 763,
    V2EncodeClientResponse = 764,
    V2DecodeAgentNotification = 765,
    V2EncodeAgentNotification = 766,
    V2DecodeProtocolLevelNotification = 767,
    V2EncodeProtocolLevelNotification = 768,
    RawJsonMember = 769,
    DefaultOptions = 770,
    ConnectMessages = 771,
    FeedMessage = 772,
    TryTakeMessage = 773,
    CloseInput = 774,
    ConnectProcess = 775,
    Send = 776,
    Notify = 777,
    TryReceive = 778,
    Receive = 779,
    Respond = 780,
    RespondError = 781,
    CancelRequest = 782,
    State = 783,
    Close = 784,
    Release = 785,
    V2Send = 786,
    V2Notify = 787,
    V2Respond = 788,
    HostReadTextFile = 789,
    HostWriteTextFile = 790,
    HostCreateTerminal = 791,
    HostTerminalOutput = 792,
    HostWaitForTerminalExit = 793,
    HostKillTerminal = 794,
    HostReleaseTerminal = 795,
    HostRunTerminalAuthentication = 796,
    HostValidateElicitation = 797,
    HostElicitationDefaults = 798,
    RegisterMcpEndpoint = 799,
    RemoveMcpEndpoint = 800,
    SendMcp = 801,
    V2SendMcp = 802,
    HostV2RunTerminalAuthentication = 803,
}
}

impl AcpIntrinsic {
    /// Operations requiring connection-owned host state rather than pure value codecs.
    #[must_use]
    pub const fn connection(self) -> bool {
        self as u16 >= 770
    }

    /// Operations retaining cooperative VM wait state.
    #[must_use]
    pub const fn waits(self) -> bool {
        matches!(
            self,
            Self::Receive | Self::Close | Self::Release | Self::HostWaitForTerminalExit
        )
    }
    /// Protocol version, decode/encode direction, and typed root for a payload codec.
    #[must_use]
    pub const fn codec(self) -> Option<(u8, bool, &'static str)> {
        let wire = self as u16;
        if wire < 741 || wire > 768 {
            return None;
        }
        let index = (wire - 741) % 14;
        let root = match index / 2 {
            0 => "ClientRequest",
            1 => "AgentResponse",
            2 => "ClientNotification",
            3 => "AgentRequest",
            4 => "ClientResponse",
            5 => "AgentNotification",
            _ => "ProtocolLevelNotification",
        };
        Some((if wire >= 755 { 2 } else { 1 }, index % 2 == 0, root))
    }

    /// Canonical public source name.
    #[must_use]
    pub fn source_name(self) -> String {
        let name = format!("{self:?}");
        if let Some(name) = name.strip_prefix("Host") {
            return format!("Std.Acp.Host.{name}");
        }
        if let Some(name) = name.strip_prefix("V2") {
            format!("Std.Acp.V2.{name}")
        } else if self.connection() {
            format!("Std.Acp.{name}")
        } else {
            format!("Std.Acp.Types.{name}")
        }
    }
}
