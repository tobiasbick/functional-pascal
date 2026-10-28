//! Exhaustive intrinsic policy at the common execution boundary.

use crate::{ConsoleIntrinsic, Intrinsic, TimeIntrinsic};

use crate::execution::vm::{VmError, worker::Worker};

impl Worker {
    pub(in crate::execution::vm) fn check_scheduling_effect(
        &self,
        intrinsic: Intrinsic,
    ) -> Result<(), VmError> {
        if self.debug_schedule.is_none() {
            return Ok(());
        }
        let denied = if self.has_synchronous_callback_boundary()
            && matches!(
                intrinsic,
                Intrinsic::Task(_) | Intrinsic::Time(TimeIntrinsic::Sleep)
            ) {
            Some("task operations inside a synchronous native callback")
        } else {
            unsupported(intrinsic)
        };
        let Some(effect) = denied else {
            return Ok(());
        };
        self.reject_scheduling_effect(effect)
    }

    pub(in crate::execution::vm) fn check_scheduling_scope(&self) -> Result<(), VmError> {
        if self.has_synchronous_callback_boundary() && self.debug_schedule.is_some() {
            return self.reject_scheduling_effect(
                "task scopes, spawning or yielding inside a synchronous native callback",
            );
        }
        Ok(())
    }

    fn has_synchronous_callback_boundary(&self) -> bool {
        self.capture_callback || (self.task_id == 0 && !self.callback_continuations.is_empty())
    }

    fn reject_scheduling_effect(&self, effect: &str) -> Result<(), VmError> {
        let Some(guard) = &self.debug_schedule else {
            return Ok(());
        };
        let error = self.runtime_error(
            fpas_diagnostics::codes::RUNTIME_SCHEDULE_UNSUPPORTED_EFFECT,
            format!("seeded scheduling cannot execute {effect}"),
            "Use ordinary debugging without --schedule-seed, or supply fixed arguments and queued text input. Move waiting task operations out of synchronous native callbacks.",
        );
        Err(guard.reject(self.task_id, error))
    }
}

fn unsupported(intrinsic: Intrinsic) -> Option<&'static str> {
    match intrinsic {
        Intrinsic::Mcp(_) => {
            if crate::intrinsic_debug_effects(intrinsic).is_debug_safe() {
                None
            } else {
                Some("Std.Ai.Mcp")
            }
        }
        Intrinsic::Server(_) => Some("Std.Net.Server"),
        Intrinsic::Random(_) => Some("Std.Math.Random"),
        Intrinsic::Crypto(_) => Some("Std.Crypto"),
        Intrinsic::Acp(_) | Intrinsic::Encoding(_) => None,
        Intrinsic::Time(TimeIntrinsic::Sleep) => None,
        Intrinsic::Time(_) => Some("wall-clock Std.Time"),
        Intrinsic::Fs(_) => Some("Std.Io.Fs"),
        Intrinsic::Env(_) => Some("Std.Os.Env"),
        Intrinsic::Proc(
            crate::ProcIntrinsic::DefaultOptions | crate::ProcIntrinsic::DefaultStartOptions,
        ) => None,
        Intrinsic::Proc(_) => Some("Std.Os.Proc"),
        Intrinsic::Net(_) => Some("Std.Net.Sockets"),
        Intrinsic::Http(_) => Some("Std.Net.Http"),
        Intrinsic::Console(
            ConsoleIntrinsic::Read
            | ConsoleIntrinsic::ReadLn
            | ConsoleIntrinsic::Write
            | ConsoleIntrinsic::WriteLn,
        ) => None,
        Intrinsic::Console(_) => {
            Some("console effects other than queued text input and captured text output")
        }
        Intrinsic::Args(crate::ArgsIntrinsic::ParamCount | crate::ArgsIntrinsic::ParamStr) => None,
        Intrinsic::Str(
            crate::StrIntrinsic::Length
            | crate::StrIntrinsic::ToUpper
            | crate::StrIntrinsic::ToLower
            | crate::StrIntrinsic::Trim
            | crate::StrIntrinsic::StartsWith
            | crate::StrIntrinsic::EndsWith
            | crate::StrIntrinsic::Substring
            | crate::StrIntrinsic::IndexOf
            | crate::StrIntrinsic::Replace
            | crate::StrIntrinsic::Split
            | crate::StrIntrinsic::Join
            | crate::StrIntrinsic::IsNumeric
            | crate::StrIntrinsic::Repeat
            | crate::StrIntrinsic::PadLeft
            | crate::StrIntrinsic::PadRight
            | crate::StrIntrinsic::PadCenter
            | crate::StrIntrinsic::SetCharAt
            | crate::StrIntrinsic::Ord
            | crate::StrIntrinsic::Chr
            | crate::StrIntrinsic::Insert
            | crate::StrIntrinsic::Delete
            | crate::StrIntrinsic::Reverse
            | crate::StrIntrinsic::TrimLeft
            | crate::StrIntrinsic::TrimRight
            | crate::StrIntrinsic::LastIndexOf
            | crate::StrIntrinsic::Format
            | crate::StrIntrinsic::Utf8Encode
            | crate::StrIntrinsic::Map
            | crate::StrIntrinsic::Filter
            | crate::StrIntrinsic::Reduce,
        ) => None,
        Intrinsic::Conv(
            crate::ConvIntrinsic::IntToStr
            | crate::ConvIntrinsic::ErrorMessage
            | crate::ConvIntrinsic::ErrorMatchesVariant
            | crate::ConvIntrinsic::RealToStr
            | crate::ConvIntrinsic::IntToReal
            | crate::ConvIntrinsic::BoolToStr
            | crate::ConvIntrinsic::IntToHex,
        ) => None,
        Intrinsic::Parse(
            crate::ParseIntrinsic::Int
            | crate::ParseIntrinsic::Real
            | crate::ParseIntrinsic::Bool
            | crate::ParseIntrinsic::HexInt,
        ) => None,
        Intrinsic::Math(
            crate::MathIntrinsic::Sqrt
            | crate::MathIntrinsic::Pow
            | crate::MathIntrinsic::Floor
            | crate::MathIntrinsic::Ceil
            | crate::MathIntrinsic::Round
            | crate::MathIntrinsic::Sin
            | crate::MathIntrinsic::Cos
            | crate::MathIntrinsic::Log
            | crate::MathIntrinsic::Min
            | crate::MathIntrinsic::Max
            | crate::MathIntrinsic::Abs
            | crate::MathIntrinsic::Tan
            | crate::MathIntrinsic::ArcSin
            | crate::MathIntrinsic::ArcCos
            | crate::MathIntrinsic::ArcTan
            | crate::MathIntrinsic::ArcTan2
            | crate::MathIntrinsic::Exp
            | crate::MathIntrinsic::Log10
            | crate::MathIntrinsic::Log2
            | crate::MathIntrinsic::Trunc
            | crate::MathIntrinsic::Frac
            | crate::MathIntrinsic::Sign
            | crate::MathIntrinsic::Clamp,
        ) => None,
        Intrinsic::Bits(
            crate::BitsIntrinsic::BitAnd
            | crate::BitsIntrinsic::BitOr
            | crate::BitsIntrinsic::BitXor
            | crate::BitsIntrinsic::BitNot
            | crate::BitsIntrinsic::ShiftLeft
            | crate::BitsIntrinsic::ShiftRight,
        ) => None,
        Intrinsic::Path(
            crate::PathIntrinsic::Join
            | crate::PathIntrinsic::BaseName
            | crate::PathIntrinsic::DirName
            | crate::PathIntrinsic::Extension
            | crate::PathIntrinsic::Normalize
            | crate::PathIntrinsic::IsAbsolute
            | crate::PathIntrinsic::RelativeTo
            | crate::PathIntrinsic::ChangeExtension,
        ) => None,
        Intrinsic::Json(crate::JsonIntrinsic::Parse | crate::JsonIntrinsic::Stringify) => None,
        Intrinsic::Toml(crate::TomlIntrinsic::Parse | crate::TomlIntrinsic::Stringify) => None,
        Intrinsic::Array(
            crate::ArrayIntrinsic::Length
            | crate::ArrayIntrinsic::Sort
            | crate::ArrayIntrinsic::SortBy
            | crate::ArrayIntrinsic::Reverse
            | crate::ArrayIntrinsic::IndexOf
            | crate::ArrayIntrinsic::Slice
            | crate::ArrayIntrinsic::Map
            | crate::ArrayIntrinsic::Filter
            | crate::ArrayIntrinsic::Reduce
            | crate::ArrayIntrinsic::Concat
            | crate::ArrayIntrinsic::Fill
            | crate::ArrayIntrinsic::Find
            | crate::ArrayIntrinsic::FindIndex
            | crate::ArrayIntrinsic::Any
            | crate::ArrayIntrinsic::All
            | crate::ArrayIntrinsic::FlatMap
            | crate::ArrayIntrinsic::ForEach,
        ) => None,
        Intrinsic::Dict(
            crate::DictIntrinsic::Length
            | crate::DictIntrinsic::Keys
            | crate::DictIntrinsic::Values
            | crate::DictIntrinsic::Remove
            | crate::DictIntrinsic::Get
            | crate::DictIntrinsic::Merge
            | crate::DictIntrinsic::Map
            | crate::DictIntrinsic::Filter
            | crate::DictIntrinsic::Reduce,
        ) => None,
        Intrinsic::Result(
            crate::ResultIntrinsic::Unwrap
            | crate::ResultIntrinsic::Map
            | crate::ResultIntrinsic::AndThen
            | crate::ResultIntrinsic::OrElse
            | crate::ResultIntrinsic::MapError,
        ) => None,
        Intrinsic::Option(
            crate::OptionIntrinsic::Unwrap
            | crate::OptionIntrinsic::Map
            | crate::OptionIntrinsic::AndThen
            | crate::OptionIntrinsic::OrElse,
        ) => None,
        Intrinsic::Task(
            crate::TaskIntrinsic::Wait
            | crate::TaskIntrinsic::WaitAll
            | crate::TaskIntrinsic::WaitAny
            | crate::TaskIntrinsic::ReceiveCase
            | crate::TaskIntrinsic::SendCase
            | crate::TaskIntrinsic::TaskCase
            | crate::TaskIntrinsic::TimerCase
            | crate::TaskIntrinsic::CancellationCase
            | crate::TaskIntrinsic::Select
            | crate::TaskIntrinsic::CloseWaitCase
            | crate::TaskIntrinsic::CreateTaskGroup
            | crate::TaskIntrinsic::StartTaskInGroup
            | crate::TaskIntrinsic::GetTaskGroupToken
            | crate::TaskIntrinsic::CancelTaskGroup
            | crate::TaskIntrinsic::CloseTaskGroup
            | crate::TaskIntrinsic::CloseTaskGroupWithTimeout
            | crate::TaskIntrinsic::TryCloseCompletedTaskGroup
            | crate::TaskIntrinsic::StartSupervisedTask
            | crate::TaskIntrinsic::WaitAnyWithTimeout
            | crate::TaskIntrinsic::WaitAnyWithCancellation
            | crate::TaskIntrinsic::CreateCancellationSource
            | crate::TaskIntrinsic::GetCancellationToken
            | crate::TaskIntrinsic::Cancel
            | crate::TaskIntrinsic::IsCancellationRequested
            | crate::TaskIntrinsic::CreateChannel
            | crate::TaskIntrinsic::Send
            | crate::TaskIntrinsic::TrySend
            | crate::TaskIntrinsic::SendWithCancellation
            | crate::TaskIntrinsic::SendWithTimeout
            | crate::TaskIntrinsic::Receive
            | crate::TaskIntrinsic::TryReceive
            | crate::TaskIntrinsic::ReceiveWithCancellation
            | crate::TaskIntrinsic::ReceiveWithTimeout
            | crate::TaskIntrinsic::CloseChannel,
        ) => None,
        Intrinsic::Test(
            crate::TestIntrinsic::ScratchDir
            | crate::TestIntrinsic::PushReadLn
            | crate::TestIntrinsic::AssertScreenLine
            | crate::TestIntrinsic::AssertScreenCell,
        ) => Some("Std.Test filesystem, input mutation or screen operations"),
        Intrinsic::Test(
            crate::TestIntrinsic::AssertTrue
            | crate::TestIntrinsic::AssertFalse
            | crate::TestIntrinsic::AssertEqualsInteger
            | crate::TestIntrinsic::Fail
            | crate::TestIntrinsic::Skip
            | crate::TestIntrinsic::AssertEqualsBoolean
            | crate::TestIntrinsic::AssertEqualsString
            | crate::TestIntrinsic::AssertEqualsReal,
        ) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_intrinsic_has_an_explicit_scheduling_policy() {
        for intrinsic in Intrinsic::all() {
            let _ = unsupported(intrinsic);
        }
        assert!(unsupported(Intrinsic::Time(TimeIntrinsic::Sleep)).is_none());
        assert!(unsupported(Intrinsic::Console(ConsoleIntrinsic::ReadLn)).is_none());
        assert!(unsupported(Intrinsic::Console(ConsoleIntrinsic::ReadKey)).is_some());
        for operation in [
            crate::PathIntrinsic::IsAbsolute,
            crate::PathIntrinsic::RelativeTo,
            crate::PathIntrinsic::ChangeExtension,
        ] {
            assert!(unsupported(Intrinsic::Path(operation)).is_none());
        }
    }
}
