//! Typed debugger command names shared by every protocol adapter.

/// One debugger operation name independent from its wire spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DebugCommand {
    /// Start protocol negotiation.
    Initialize,
    /// Start target execution.
    Launch,
    /// Stop an active target.
    Pause,
    /// Continue active target execution.
    Continue,
    /// Advance into the next instruction.
    StepInto,
    /// Advance over the next instruction.
    StepOver,
    /// Advance out of the current frame.
    StepOut,
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
    DataBreakpointsReplace,
    BreakpointSet,
    BreakpointClear,
    FunctionBreakpointsReplace,
    RuntimeFailuresReplace,
    Tasks,
    TaskPause,
    TaskResume,
    TaskCancel,
    TaskCreate,
    TaskRestart,
    IoInput,
    IoEof,
    IoCancel,
    Stack,
    Scopes,
    Variables,
    Evaluate,
    VariableSet,
    ExpressionSet,
    DictionaryInsert,
    DictionaryRemove,
    DictionaryReplaceKey,
    ArrayInsert,
    ArrayRemove,
    StringReplaceCharacter,
    FrameReturn,
    FrameRestart,
    InstructionSet,
    LocationDescribe,
    RecordingDescribe,
    TaskResultReplace,
    VariantDescribe,
    VariantConstruct,
    StorageInitialize,
    EvaluateCancel,
    Disconnect,
    /// A command not defined by the current engine version.
    Unknown(String),
}

impl DebugCommand {
    /// Parse a command name supplied by a protocol adapter.
    #[must_use]
    pub(crate) fn from_name(name: &str) -> Self {
        match name {
            "initialize" => Self::Initialize,
            "launch" => Self::Launch,
            "pause" => Self::Pause,
            "continue" => Self::Continue,
            "step_into" => Self::StepInto,
            "step_over" => Self::StepOver,
            "step_out" => Self::StepOut,
            "attach" => Self::Attach,
            "step_back" => Self::StepBack,
            "reverse_continue" => Self::ReverseContinue,
            "replay" => Self::Replay,
            "reload" => Self::Reload,
            "image.replace" => Self::ImageReplace,
            "image.rollback" => Self::ImageRollback,
            "reload.classify" => Self::ReloadClassify,
            "record" => Self::Record,
            "data_breakpoint.set" => Self::DataBreakpointSet,
            "data_breakpoints.replace" => Self::DataBreakpointsReplace,
            "breakpoint.set" => Self::BreakpointSet,
            "breakpoint.clear" => Self::BreakpointClear,
            "function_breakpoints.replace" => Self::FunctionBreakpointsReplace,
            "runtime_failures.replace" => Self::RuntimeFailuresReplace,
            "tasks" => Self::Tasks,
            "task.pause" => Self::TaskPause,
            "task.resume" => Self::TaskResume,
            "task.cancel" => Self::TaskCancel,
            "task.create" => Self::TaskCreate,
            "task.restart" => Self::TaskRestart,
            "io.input" => Self::IoInput,
            "io.eof" => Self::IoEof,
            "io.cancel" => Self::IoCancel,
            "stack" => Self::Stack,
            "scopes" => Self::Scopes,
            "variables" => Self::Variables,
            "evaluate" => Self::Evaluate,
            "variable.set" => Self::VariableSet,
            "expression.set" => Self::ExpressionSet,
            "dictionary.insert" => Self::DictionaryInsert,
            "dictionary.remove" => Self::DictionaryRemove,
            "dictionary.replace_key" => Self::DictionaryReplaceKey,
            "array.insert" => Self::ArrayInsert,
            "array.remove" => Self::ArrayRemove,
            "string.replace_character" => Self::StringReplaceCharacter,
            "frame.return" => Self::FrameReturn,
            "frame.restart" => Self::FrameRestart,
            "instruction.set" => Self::InstructionSet,
            "location.describe" => Self::LocationDescribe,
            "recording.describe" => Self::RecordingDescribe,
            "task.result.replace" => Self::TaskResultReplace,
            "variant.describe" => Self::VariantDescribe,
            "variant.construct" => Self::VariantConstruct,
            "storage.initialize" => Self::StorageInitialize,
            "evaluate.cancel" => Self::EvaluateCancel,
            "disconnect" => Self::Disconnect,
            _ => Self::Unknown(name.to_owned()),
        }
    }

    /// Return the stable protocol spelling while legacy commands are migrated.
    #[must_use]
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Initialize => "initialize",
            Self::Launch => "launch",
            Self::Pause => "pause",
            Self::Continue => "continue",
            Self::StepInto => "step_into",
            Self::StepOver => "step_over",
            Self::StepOut => "step_out",
            Self::Attach => "attach",
            Self::StepBack => "step_back",
            Self::ReverseContinue => "reverse_continue",
            Self::Replay => "replay",
            Self::Reload => "reload",
            Self::ImageReplace => "image.replace",
            Self::ImageRollback => "image.rollback",
            Self::ReloadClassify => "reload.classify",
            Self::Record => "record",
            Self::DataBreakpointSet => "data_breakpoint.set",
            Self::DataBreakpointsReplace => "data_breakpoints.replace",
            Self::BreakpointSet => "breakpoint.set",
            Self::BreakpointClear => "breakpoint.clear",
            Self::FunctionBreakpointsReplace => "function_breakpoints.replace",
            Self::RuntimeFailuresReplace => "runtime_failures.replace",
            Self::Tasks => "tasks",
            Self::TaskPause => "task.pause",
            Self::TaskResume => "task.resume",
            Self::TaskCancel => "task.cancel",
            Self::TaskCreate => "task.create",
            Self::TaskRestart => "task.restart",
            Self::IoInput => "io.input",
            Self::IoEof => "io.eof",
            Self::IoCancel => "io.cancel",
            Self::Stack => "stack",
            Self::Scopes => "scopes",
            Self::Variables => "variables",
            Self::Evaluate => "evaluate",
            Self::VariableSet => "variable.set",
            Self::ExpressionSet => "expression.set",
            Self::DictionaryInsert => "dictionary.insert",
            Self::DictionaryRemove => "dictionary.remove",
            Self::DictionaryReplaceKey => "dictionary.replace_key",
            Self::ArrayInsert => "array.insert",
            Self::ArrayRemove => "array.remove",
            Self::StringReplaceCharacter => "string.replace_character",
            Self::FrameReturn => "frame.return",
            Self::FrameRestart => "frame.restart",
            Self::InstructionSet => "instruction.set",
            Self::LocationDescribe => "location.describe",
            Self::RecordingDescribe => "recording.describe",
            Self::TaskResultReplace => "task.result.replace",
            Self::VariantDescribe => "variant.describe",
            Self::VariantConstruct => "variant.construct",
            Self::StorageInitialize => "storage.initialize",
            Self::EvaluateCancel => "evaluate.cancel",
            Self::Disconnect => "disconnect",
            Self::Unknown(name) => name,
        }
    }
}

impl From<&str> for DebugCommand {
    fn from(name: &str) -> Self {
        Self::from_name(name)
    }
}

#[cfg(test)]
mod tests {
    use super::DebugCommand;

    #[test]
    fn core_lifecycle_commands_use_typed_variants() {
        for name in [
            "initialize",
            "launch",
            "continue",
            "pause",
            "step_into",
            "step_over",
            "step_out",
        ] {
            let command = DebugCommand::from_name(name);
            assert!(!matches!(command, DebugCommand::Unknown(_)));
            assert_eq!(command.name(), name);
        }
    }

    #[test]
    fn unrecognized_command_preserves_its_wire_name_for_diagnostics() {
        let command = DebugCommand::from_name("future.command");
        assert_eq!(command.name(), "future.command");
    }
}
