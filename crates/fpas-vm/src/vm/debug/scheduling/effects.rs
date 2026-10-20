//! Exhaustive intrinsic policy at the common execution boundary.

use fpas_bytecode::{ConsoleIntrinsic, Intrinsic, TimeIntrinsic};

use crate::vm::{VmError, worker::Worker};

impl Worker {
    pub(in crate::vm) fn check_scheduling_effect(
        &self,
        intrinsic: Intrinsic,
    ) -> Result<(), VmError> {
        if self.debug_schedule.is_none() {
            return Ok(());
        }
        let denied = if self.capture_callback
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

    pub(in crate::vm) fn check_scheduling_scope(&self) -> Result<(), VmError> {
        if self.capture_callback && self.debug_schedule.is_some() {
            return self.reject_scheduling_effect(
                "task scopes, spawning or yielding inside a synchronous native callback",
            );
        }
        Ok(())
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
        Intrinsic::Server(_) => Some("Std.Server"),
        Intrinsic::Random(_) => Some("Std.Random"),
        Intrinsic::Crypto(_) => Some("Std.Crypto"),
        Intrinsic::Time(TimeIntrinsic::Sleep) => None,
        Intrinsic::Time(_) => Some("wall-clock Std.Time"),
        Intrinsic::Fs(_) => Some("Std.Fs"),
        Intrinsic::Env(_) => Some("Std.Env"),
        Intrinsic::Proc(_) => Some("Std.Proc"),
        Intrinsic::Net(_) => Some("Std.Net"),
        Intrinsic::Http(_) => Some("Std.Http"),
        Intrinsic::Console(
            ConsoleIntrinsic::Read
            | ConsoleIntrinsic::ReadLn
            | ConsoleIntrinsic::Write
            | ConsoleIntrinsic::WriteLn,
        ) => None,
        Intrinsic::Console(_) => {
            Some("console effects other than queued text input and captured text output")
        }
        Intrinsic::Args(
            fpas_bytecode::ArgsIntrinsic::ParamCount | fpas_bytecode::ArgsIntrinsic::ParamStr,
        ) => None,
        Intrinsic::Str(
            fpas_bytecode::StrIntrinsic::Length
            | fpas_bytecode::StrIntrinsic::ToUpper
            | fpas_bytecode::StrIntrinsic::ToLower
            | fpas_bytecode::StrIntrinsic::Trim
            | fpas_bytecode::StrIntrinsic::StartsWith
            | fpas_bytecode::StrIntrinsic::EndsWith
            | fpas_bytecode::StrIntrinsic::Substring
            | fpas_bytecode::StrIntrinsic::IndexOf
            | fpas_bytecode::StrIntrinsic::Replace
            | fpas_bytecode::StrIntrinsic::Split
            | fpas_bytecode::StrIntrinsic::Join
            | fpas_bytecode::StrIntrinsic::IsNumeric
            | fpas_bytecode::StrIntrinsic::Repeat
            | fpas_bytecode::StrIntrinsic::PadLeft
            | fpas_bytecode::StrIntrinsic::PadRight
            | fpas_bytecode::StrIntrinsic::PadCenter
            | fpas_bytecode::StrIntrinsic::SetCharAt
            | fpas_bytecode::StrIntrinsic::Ord
            | fpas_bytecode::StrIntrinsic::Chr
            | fpas_bytecode::StrIntrinsic::Insert
            | fpas_bytecode::StrIntrinsic::Delete
            | fpas_bytecode::StrIntrinsic::Reverse
            | fpas_bytecode::StrIntrinsic::TrimLeft
            | fpas_bytecode::StrIntrinsic::TrimRight
            | fpas_bytecode::StrIntrinsic::LastIndexOf
            | fpas_bytecode::StrIntrinsic::Format
            | fpas_bytecode::StrIntrinsic::Utf8Encode
            | fpas_bytecode::StrIntrinsic::Map
            | fpas_bytecode::StrIntrinsic::Filter
            | fpas_bytecode::StrIntrinsic::Reduce,
        ) => None,
        Intrinsic::Conv(
            fpas_bytecode::ConvIntrinsic::IntToStr
            | fpas_bytecode::ConvIntrinsic::ErrorMessage
            | fpas_bytecode::ConvIntrinsic::ErrorMatchesVariant
            | fpas_bytecode::ConvIntrinsic::RealToStr
            | fpas_bytecode::ConvIntrinsic::IntToReal
            | fpas_bytecode::ConvIntrinsic::BoolToStr
            | fpas_bytecode::ConvIntrinsic::IntToHex,
        ) => None,
        Intrinsic::Parse(
            fpas_bytecode::ParseIntrinsic::Int
            | fpas_bytecode::ParseIntrinsic::Real
            | fpas_bytecode::ParseIntrinsic::Bool
            | fpas_bytecode::ParseIntrinsic::HexInt,
        ) => None,
        Intrinsic::Math(
            fpas_bytecode::MathIntrinsic::Sqrt
            | fpas_bytecode::MathIntrinsic::Pow
            | fpas_bytecode::MathIntrinsic::Floor
            | fpas_bytecode::MathIntrinsic::Ceil
            | fpas_bytecode::MathIntrinsic::Round
            | fpas_bytecode::MathIntrinsic::Sin
            | fpas_bytecode::MathIntrinsic::Cos
            | fpas_bytecode::MathIntrinsic::Log
            | fpas_bytecode::MathIntrinsic::Min
            | fpas_bytecode::MathIntrinsic::Max
            | fpas_bytecode::MathIntrinsic::Abs
            | fpas_bytecode::MathIntrinsic::Tan
            | fpas_bytecode::MathIntrinsic::ArcSin
            | fpas_bytecode::MathIntrinsic::ArcCos
            | fpas_bytecode::MathIntrinsic::ArcTan
            | fpas_bytecode::MathIntrinsic::ArcTan2
            | fpas_bytecode::MathIntrinsic::Exp
            | fpas_bytecode::MathIntrinsic::Log10
            | fpas_bytecode::MathIntrinsic::Log2
            | fpas_bytecode::MathIntrinsic::Trunc
            | fpas_bytecode::MathIntrinsic::Frac
            | fpas_bytecode::MathIntrinsic::Sign
            | fpas_bytecode::MathIntrinsic::Clamp,
        ) => None,
        Intrinsic::Bits(
            fpas_bytecode::BitsIntrinsic::BitAnd
            | fpas_bytecode::BitsIntrinsic::BitOr
            | fpas_bytecode::BitsIntrinsic::BitXor
            | fpas_bytecode::BitsIntrinsic::BitNot
            | fpas_bytecode::BitsIntrinsic::ShiftLeft
            | fpas_bytecode::BitsIntrinsic::ShiftRight,
        ) => None,
        Intrinsic::Path(
            fpas_bytecode::PathIntrinsic::Join
            | fpas_bytecode::PathIntrinsic::BaseName
            | fpas_bytecode::PathIntrinsic::DirName
            | fpas_bytecode::PathIntrinsic::Extension
            | fpas_bytecode::PathIntrinsic::Normalize
            | fpas_bytecode::PathIntrinsic::IsAbsolute
            | fpas_bytecode::PathIntrinsic::RelativeTo
            | fpas_bytecode::PathIntrinsic::ChangeExtension,
        ) => None,
        Intrinsic::Json(
            fpas_bytecode::JsonIntrinsic::Parse | fpas_bytecode::JsonIntrinsic::Stringify,
        ) => None,
        Intrinsic::Toml(
            fpas_bytecode::TomlIntrinsic::Parse | fpas_bytecode::TomlIntrinsic::Stringify,
        ) => None,
        Intrinsic::Array(
            fpas_bytecode::ArrayIntrinsic::Length
            | fpas_bytecode::ArrayIntrinsic::Sort
            | fpas_bytecode::ArrayIntrinsic::SortBy
            | fpas_bytecode::ArrayIntrinsic::Reverse
            | fpas_bytecode::ArrayIntrinsic::IndexOf
            | fpas_bytecode::ArrayIntrinsic::Slice
            | fpas_bytecode::ArrayIntrinsic::Map
            | fpas_bytecode::ArrayIntrinsic::Filter
            | fpas_bytecode::ArrayIntrinsic::Reduce
            | fpas_bytecode::ArrayIntrinsic::Concat
            | fpas_bytecode::ArrayIntrinsic::Fill
            | fpas_bytecode::ArrayIntrinsic::Find
            | fpas_bytecode::ArrayIntrinsic::FindIndex
            | fpas_bytecode::ArrayIntrinsic::Any
            | fpas_bytecode::ArrayIntrinsic::All
            | fpas_bytecode::ArrayIntrinsic::FlatMap
            | fpas_bytecode::ArrayIntrinsic::ForEach,
        ) => None,
        Intrinsic::Dict(
            fpas_bytecode::DictIntrinsic::Length
            | fpas_bytecode::DictIntrinsic::Keys
            | fpas_bytecode::DictIntrinsic::Values
            | fpas_bytecode::DictIntrinsic::Remove
            | fpas_bytecode::DictIntrinsic::Get
            | fpas_bytecode::DictIntrinsic::Merge
            | fpas_bytecode::DictIntrinsic::Map
            | fpas_bytecode::DictIntrinsic::Filter
            | fpas_bytecode::DictIntrinsic::Reduce,
        ) => None,
        Intrinsic::Result(
            fpas_bytecode::ResultIntrinsic::Unwrap
            | fpas_bytecode::ResultIntrinsic::Map
            | fpas_bytecode::ResultIntrinsic::AndThen
            | fpas_bytecode::ResultIntrinsic::OrElse
            | fpas_bytecode::ResultIntrinsic::MapError,
        ) => None,
        Intrinsic::Option(
            fpas_bytecode::OptionIntrinsic::Unwrap
            | fpas_bytecode::OptionIntrinsic::Map
            | fpas_bytecode::OptionIntrinsic::AndThen
            | fpas_bytecode::OptionIntrinsic::OrElse,
        ) => None,
        Intrinsic::Task(
            fpas_bytecode::TaskIntrinsic::Wait
            | fpas_bytecode::TaskIntrinsic::WaitAll
            | fpas_bytecode::TaskIntrinsic::WaitAny
            | fpas_bytecode::TaskIntrinsic::ReceiveCase
            | fpas_bytecode::TaskIntrinsic::SendCase
            | fpas_bytecode::TaskIntrinsic::TaskCase
            | fpas_bytecode::TaskIntrinsic::TimerCase
            | fpas_bytecode::TaskIntrinsic::CancellationCase
            | fpas_bytecode::TaskIntrinsic::Select
            | fpas_bytecode::TaskIntrinsic::CloseWaitCase
            | fpas_bytecode::TaskIntrinsic::CreateTaskGroup
            | fpas_bytecode::TaskIntrinsic::StartTaskInGroup
            | fpas_bytecode::TaskIntrinsic::GetTaskGroupToken
            | fpas_bytecode::TaskIntrinsic::CancelTaskGroup
            | fpas_bytecode::TaskIntrinsic::CloseTaskGroup
            | fpas_bytecode::TaskIntrinsic::CloseTaskGroupWithTimeout
            | fpas_bytecode::TaskIntrinsic::TryCloseCompletedTaskGroup
            | fpas_bytecode::TaskIntrinsic::StartSupervisedTask
            | fpas_bytecode::TaskIntrinsic::WaitAnyWithTimeout
            | fpas_bytecode::TaskIntrinsic::WaitAnyWithCancellation
            | fpas_bytecode::TaskIntrinsic::CreateCancellationSource
            | fpas_bytecode::TaskIntrinsic::GetCancellationToken
            | fpas_bytecode::TaskIntrinsic::Cancel
            | fpas_bytecode::TaskIntrinsic::IsCancellationRequested
            | fpas_bytecode::TaskIntrinsic::CreateChannel
            | fpas_bytecode::TaskIntrinsic::Send
            | fpas_bytecode::TaskIntrinsic::TrySend
            | fpas_bytecode::TaskIntrinsic::SendWithCancellation
            | fpas_bytecode::TaskIntrinsic::SendWithTimeout
            | fpas_bytecode::TaskIntrinsic::Receive
            | fpas_bytecode::TaskIntrinsic::TryReceive
            | fpas_bytecode::TaskIntrinsic::ReceiveWithCancellation
            | fpas_bytecode::TaskIntrinsic::ReceiveWithTimeout
            | fpas_bytecode::TaskIntrinsic::CloseChannel,
        ) => None,
        Intrinsic::Test(
            fpas_bytecode::TestIntrinsic::ScratchDir
            | fpas_bytecode::TestIntrinsic::PushReadLn
            | fpas_bytecode::TestIntrinsic::AssertScreenLine
            | fpas_bytecode::TestIntrinsic::AssertScreenCell,
        ) => Some("Std.Test filesystem, input mutation or screen operations"),
        Intrinsic::Test(
            fpas_bytecode::TestIntrinsic::AssertTrue
            | fpas_bytecode::TestIntrinsic::AssertFalse
            | fpas_bytecode::TestIntrinsic::AssertEqualsInteger
            | fpas_bytecode::TestIntrinsic::Fail
            | fpas_bytecode::TestIntrinsic::Skip
            | fpas_bytecode::TestIntrinsic::AssertEqualsBoolean
            | fpas_bytecode::TestIntrinsic::AssertEqualsString
            | fpas_bytecode::TestIntrinsic::AssertEqualsReal,
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
            fpas_bytecode::PathIntrinsic::IsAbsolute,
            fpas_bytecode::PathIntrinsic::RelativeTo,
            fpas_bytecode::PathIntrinsic::ChangeExtension,
        ] {
            assert!(unsupported(Intrinsic::Path(operation)).is_none());
        }
    }
}
