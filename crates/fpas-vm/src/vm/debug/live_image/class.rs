//! Named live-image update classes for compatibility classification.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

/// One named difference between the live executable and a candidate image.
///
/// Classification does not replace the live image. Only
/// [`Unchanged`](Self::Unchanged) and
/// [`InactiveFunctionBody`](Self::InactiveFunctionBody) are accepted by the
/// proven subset.
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveImageUpdateClass {
    /// Candidate matches the live image for execution identity.
    Unchanged,
    /// Only inactive function bodies differ; signatures and layouts match.
    InactiveFunctionBody,
    /// A function currently on a stack has a different body.
    ActiveFunctionBody,
    /// Record field, property, or method layout differs.
    RecordLayout,
    /// Enum type or variant layout differs.
    EnumLayout,
    /// Global slot names, types, or mutability differ.
    GlobalLayout,
    /// Capture counts, capture sources, or lexical owners differ.
    ClosureCapture,
    /// Task-spawning flags differ; live task identities are not remapped.
    TaskIdentity,
    /// Function names, arity, or return convention differ.
    FunctionSet,
    /// A new capturing function appears; `UMB-10B` stays blocked.
    AnonymousClosure,
    /// The entry function identity differs.
    EntryPoint,
    /// Source maps, sequence points, or debug types differ without a body change.
    DebugMetadata,
}

impl LiveImageUpdateClass {
    /// Classes the proven subset may accept. They are not applied until later children.
    pub const ACCEPTED: &'static [Self] = &[Self::Unchanged, Self::InactiveFunctionBody];

    /// Classes the proven subset rejects before any live-image replacement.
    pub const REJECTED: &'static [Self] = &[
        Self::ActiveFunctionBody,
        Self::RecordLayout,
        Self::EnumLayout,
        Self::GlobalLayout,
        Self::ClosureCapture,
        Self::TaskIdentity,
        Self::FunctionSet,
        Self::AnonymousClosure,
        Self::EntryPoint,
        Self::DebugMetadata,
    ];

    /// Protocol and test identifier for this class.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::InactiveFunctionBody => "inactive_function_body",
            Self::ActiveFunctionBody => "active_function_body",
            Self::RecordLayout => "record_layout",
            Self::EnumLayout => "enum_layout",
            Self::GlobalLayout => "global_layout",
            Self::ClosureCapture => "closure_capture",
            Self::TaskIdentity => "task_identity",
            Self::FunctionSet => "function_set",
            Self::AnonymousClosure => "anonymous_closure",
            Self::EntryPoint => "entry_point",
            Self::DebugMetadata => "debug_metadata",
        }
    }

    /// Whether this class may later commit without a compatibility reject.
    #[must_use]
    pub const fn is_accepted(self) -> bool {
        matches!(self, Self::Unchanged | Self::InactiveFunctionBody)
    }
}

/// Result of comparing a candidate executable with the live image.
///
/// `accepted` follows [`LiveImageUpdateClass::is_accepted`]. Classification
/// never commits a replacement.
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiveImageClassification {
    /// Named update class for the candidate.
    pub class: LiveImageUpdateClass,
    /// Whether the proven subset treats this class as compatible.
    pub accepted: bool,
}

impl LiveImageClassification {
    pub(super) const fn new(class: LiveImageUpdateClass) -> Self {
        Self {
            class,
            accepted: class.is_accepted(),
        }
    }
}
