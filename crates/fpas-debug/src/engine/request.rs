//! Typed debug engine operations delivered by protocol adapters.

use super::command::DebugCommand;

/// One debugger operation with domain arguments, independent from wire spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DebugOp {
    Initialize,
    Launch {
        stop_on_entry: bool,
    },
    Pause,
    Continue,
    StepInto {
        task_id: Option<u64>,
    },
    StepOver {
        task_id: Option<u64>,
    },
    StepOut {
        task_id: Option<u64>,
    },
    Attach,
    StepBack,
    ReverseContinue,
    Replay,
    Reload,
    ImageReplace,
    ImageRollback,
    ReloadClassify,
    Record,
    DataBreakpointSet,
    DataBreakpointsReplace {
        breakpoints: Vec<DataBreakpointOp>,
    },
    BreakpointSet {
        source: String,
        line: u32,
        column: Option<u32>,
        assign: Option<AssignOp>,
        condition: Option<String>,
        hit_condition: Option<String>,
        log_message: Option<String>,
    },
    BreakpointClear {
        breakpoint_id: u64,
    },
    FunctionBreakpointsReplace {
        breakpoints: Vec<FunctionBreakpointOp>,
    },
    RuntimeFailuresReplace {
        filters: Vec<String>,
    },
    Tasks {
        start: usize,
        count: usize,
    },
    TaskPause {
        task_id: u64,
    },
    TaskResume {
        task_id: u64,
    },
    TaskCancel {
        task_id: u64,
    },
    TaskCreate,
    TaskRestart {
        task_id: Option<u64>,
    },
    IoInput {
        text: String,
    },
    IoEof,
    IoCancel,
    Stack {
        start: usize,
        count: usize,
        task_id: Option<u64>,
    },
    Scopes {
        frame_id: u64,
    },
    Variables {
        variables_reference: u64,
        start: usize,
        count: usize,
    },
    Evaluate {
        expression: String,
        frame_id: Option<u64>,
        async_eval: bool,
    },
    VariableSet {
        variables_reference: u64,
        name: String,
        expression: String,
    },
    ExpressionSet {
        target: String,
        expression: String,
        frame_id: Option<u64>,
    },
    DictionaryInsert {
        target: String,
        key: String,
        expression: String,
        frame_id: Option<u64>,
    },
    DictionaryRemove {
        target: String,
        key: String,
        frame_id: Option<u64>,
    },
    DictionaryReplaceKey {
        target: String,
        key: String,
        new_key: String,
        frame_id: Option<u64>,
    },
    ArrayInsert {
        target: String,
        index: String,
        expression: String,
        frame_id: Option<u64>,
    },
    ArrayRemove {
        target: String,
        index: String,
        frame_id: Option<u64>,
    },
    StringReplaceCharacter {
        target: String,
        index: String,
        expression: String,
        frame_id: Option<u64>,
    },
    FrameReturn {
        frame_id: u64,
        expression: Option<String>,
    },
    FrameRestart {
        frame_id: u64,
    },
    InstructionSet {
        frame_id: Option<u64>,
        instruction: Option<u32>,
    },
    LocationDescribe {
        variables_reference: u64,
        name: String,
    },
    RecordingDescribe,
    TaskResultReplace {
        task_id: u64,
        expression: Option<String>,
        frame_id: Option<u64>,
    },
    VariantDescribe {
        target: String,
        frame_id: Option<u64>,
    },
    VariantConstruct {
        target: String,
        variant: String,
        fields: Vec<(String, String)>,
        frame_id: Option<u64>,
    },
    StorageInitialize {
        target: String,
        initializer: String,
        expression: String,
        frame_id: Option<u64>,
    },
    EvaluateCancel,
    Disconnect,
    Unknown(String),
}

/// Assign-on-hit payload owned by the debug engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AssignOp {
    pub identity: fpas_vm::DebugDataLocationIdentity,
    pub expression: String,
}

/// One function breakpoint in a replace request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FunctionBreakpointOp {
    pub name: String,
    pub condition: Option<String>,
    pub hit_condition: Option<String>,
}

/// One data breakpoint in a replace request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DataBreakpointOp {
    pub identity: fpas_vm::DebugDataLocationIdentity,
    pub access: fpas_vm::DataBreakpointAccess,
    pub assign: Option<AssignOp>,
}

impl DebugOp {
    /// Command name this operation reports on responses.
    #[must_use]
    pub(crate) fn command(&self) -> DebugCommand {
        match self {
            Self::Initialize => DebugCommand::Initialize,
            Self::Launch { .. } => DebugCommand::Launch,
            Self::Pause => DebugCommand::Pause,
            Self::Continue => DebugCommand::Continue,
            Self::StepInto { .. } => DebugCommand::StepInto,
            Self::StepOver { .. } => DebugCommand::StepOver,
            Self::StepOut { .. } => DebugCommand::StepOut,
            Self::Attach => DebugCommand::Attach,
            Self::StepBack => DebugCommand::StepBack,
            Self::ReverseContinue => DebugCommand::ReverseContinue,
            Self::Replay => DebugCommand::Replay,
            Self::Reload => DebugCommand::Reload,
            Self::ImageReplace => DebugCommand::ImageReplace,
            Self::ImageRollback => DebugCommand::ImageRollback,
            Self::ReloadClassify => DebugCommand::ReloadClassify,
            Self::Record => DebugCommand::Record,
            Self::DataBreakpointSet => DebugCommand::DataBreakpointSet,
            Self::DataBreakpointsReplace { .. } => DebugCommand::DataBreakpointsReplace,
            Self::BreakpointSet { .. } => DebugCommand::BreakpointSet,
            Self::BreakpointClear { .. } => DebugCommand::BreakpointClear,
            Self::FunctionBreakpointsReplace { .. } => DebugCommand::FunctionBreakpointsReplace,
            Self::RuntimeFailuresReplace { .. } => DebugCommand::RuntimeFailuresReplace,
            Self::Tasks { .. } => DebugCommand::Tasks,
            Self::TaskPause { .. } => DebugCommand::TaskPause,
            Self::TaskResume { .. } => DebugCommand::TaskResume,
            Self::TaskCancel { .. } => DebugCommand::TaskCancel,
            Self::TaskCreate => DebugCommand::TaskCreate,
            Self::TaskRestart { .. } => DebugCommand::TaskRestart,
            Self::IoInput { .. } => DebugCommand::IoInput,
            Self::IoEof => DebugCommand::IoEof,
            Self::IoCancel => DebugCommand::IoCancel,
            Self::Stack { .. } => DebugCommand::Stack,
            Self::Scopes { .. } => DebugCommand::Scopes,
            Self::Variables { .. } => DebugCommand::Variables,
            Self::Evaluate { .. } => DebugCommand::Evaluate,
            Self::VariableSet { .. } => DebugCommand::VariableSet,
            Self::ExpressionSet { .. } => DebugCommand::ExpressionSet,
            Self::DictionaryInsert { .. } => DebugCommand::DictionaryInsert,
            Self::DictionaryRemove { .. } => DebugCommand::DictionaryRemove,
            Self::DictionaryReplaceKey { .. } => DebugCommand::DictionaryReplaceKey,
            Self::ArrayInsert { .. } => DebugCommand::ArrayInsert,
            Self::ArrayRemove { .. } => DebugCommand::ArrayRemove,
            Self::StringReplaceCharacter { .. } => DebugCommand::StringReplaceCharacter,
            Self::FrameReturn { .. } => DebugCommand::FrameReturn,
            Self::FrameRestart { .. } => DebugCommand::FrameRestart,
            Self::InstructionSet { .. } => DebugCommand::InstructionSet,
            Self::LocationDescribe { .. } => DebugCommand::LocationDescribe,
            Self::RecordingDescribe => DebugCommand::RecordingDescribe,
            Self::TaskResultReplace { .. } => DebugCommand::TaskResultReplace,
            Self::VariantDescribe { .. } => DebugCommand::VariantDescribe,
            Self::VariantConstruct { .. } => DebugCommand::VariantConstruct,
            Self::StorageInitialize { .. } => DebugCommand::StorageInitialize,
            Self::EvaluateCancel => DebugCommand::EvaluateCancel,
            Self::Disconnect => DebugCommand::Disconnect,
            Self::Unknown(name) => DebugCommand::Unknown(name.clone()),
        }
    }
}

/// A validated debugger request delivered to the engine.
#[derive(Debug, Clone)]
pub(crate) struct DebugRequest {
    /// Correlation identifier supplied by the adapter.
    pub id: u64,
    /// Requested debugger operation.
    pub op: DebugOp,
}

impl DebugRequest {
    /// Construct a typed engine request.
    #[must_use]
    pub(crate) fn new(id: u64, op: DebugOp) -> Self {
        Self { id, op }
    }

    #[must_use]
    pub(crate) fn command(&self) -> DebugCommand {
        self.op.command()
    }
}
