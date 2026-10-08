//! Canonical `Std.*` call names to stable intrinsic wire identifiers.

use fpas_bytecode::{
    ArgsIntrinsic, ArrayIntrinsic, BitsIntrinsic, ConsoleIntrinsic, ConvIntrinsic, CryptoIntrinsic,
    DictIntrinsic, EnvIntrinsic, FsIntrinsic, HttpIntrinsic, Intrinsic, JsonIntrinsic,
    MathIntrinsic, NetIntrinsic, OptionIntrinsic, ParseIntrinsic, PathIntrinsic, ProcIntrinsic,
    RandomIntrinsic, ResultIntrinsic, ServerIntrinsic, StrIntrinsic, TaskIntrinsic, TestIntrinsic,
    TimeIntrinsic, TomlIntrinsic,
};
use fpas_sema::Ty;

macro_rules! family {
    ($member:expr, $wrapper:ident, $kind:ident, [$($variant:ident),+ $(,)?]) => {
        match $member {
            $(stringify!($variant) => Some(Intrinsic::$wrapper($kind::$variant)),)+
            _ => None,
        }
    };
}

/// Resolve one semantically validated canonical standard-library call.
#[must_use]
pub(crate) fn resolve(name: &str, first_argument: Option<&Ty>) -> Option<Intrinsic> {
    if let Some(entry) = fpas_sema::native_operation_by_implementation(name)
        && entry.lowering == fpas_sema::NativeLowering::IsEmpty
    {
        return resolve(&name.replace("IsEmpty", "Length"), first_argument);
    }
    if name == "Std.Net.Utf8.EncodeBytes" {
        return Some(Intrinsic::Str(StrIntrinsic::Utf8Encode));
    }
    let remainder = name.strip_prefix("Std.")?;
    let (unit, member) = remainder.split_once('.')?;
    match unit {
        "Args" => family!(member, Args, ArgsIntrinsic, [ParamCount, ParamStr]),
        "Console" => resolve_console(member),
        "Str" => resolve_str(member),
        "Conv" => family!(
            member,
            Conv,
            ConvIntrinsic,
            [
                IntToStr, StrToInt, RealToStr, StrToReal, IntToReal, BoolToStr, StrToBool,
                IntToHex, HexToInt,
            ]
        ),
        "Crypto" => family!(member, Crypto, CryptoIntrinsic, [RandomBytes, RandomInt]),
        "Parse" => family!(member, Parse, ParseIntrinsic, [TryInt, TryReal, TryBool]),
        "Math" => resolve_math(member),
        "Bits" => family!(
            member,
            Bits,
            BitsIntrinsic,
            [BitAnd, BitOr, BitXor, BitNot, ShiftLeft, ShiftRight]
        ),
        "Net" => resolve_net(member),
        "Http" => resolve_http(member),
        "Random" => family!(
            member,
            Random,
            RandomIntrinsic,
            [Random, RandomInt, Randomize, SetSeed]
        ),
        "Arrays" => resolve_array(member),
        "Dictionaries" => family!(
            member,
            Dict,
            DictIntrinsic,
            [
                Length,
                ContainsKey,
                Keys,
                Values,
                Remove,
                Get,
                Merge,
                Map,
                Filter,
                Reduce,
            ]
        ),
        "Env" => family!(member, Env, EnvIntrinsic, [Get, Exists]),
        "Path" => family!(
            member,
            Path,
            PathIntrinsic,
            [Join, BaseName, DirName, Extension, Normalize]
        ),
        "Server" => family!(
            member,
            Server,
            ServerIntrinsic,
            [
                CreateLifetime,
                GetWorkGroup,
                GetStopToken,
                IsReady,
                RequestStop,
                RemainingMillis,
                OwnListener,
                FinishShutdown,
                ObserveSignals,
                ShutdownErrors
            ]
        ),
        "Proc" => family!(
            member,
            Proc,
            ProcIntrinsic,
            [Run, CurrentExecutable, RunCapture]
        ),
        "Fs" => family!(
            member,
            Fs,
            FsIntrinsic,
            [
                ReadText,
                WriteText,
                WriteTextAtomic,
                DeleteFile,
                Exists,
                IsFile,
                IsDir,
                CreateDir,
                CreateDirAll,
                Glob,
                ReadDir,
            ]
        ),
        "Json" => family!(member, Json, JsonIntrinsic, [Parse, Stringify]),
        "Results" => family!(
            member,
            Result,
            ResultIntrinsic,
            [Unwrap, UnwrapOr, IsOk, IsError, Map, AndThen, OrElse]
        ),
        "Options" => family!(
            member,
            Option,
            OptionIntrinsic,
            [Unwrap, UnwrapOr, IsSome, IsNone, Map, AndThen, OrElse]
        ),
        "Tasks" => family!(
            member,
            Task,
            TaskIntrinsic,
            [
                Wait,
                WaitAll,
                WaitAny,
                ReceiveCase,
                SendCase,
                TaskCase,
                TimerCase,
                CancellationCase,
                Select,
                CreateTaskGroup,
                StartTaskInGroup,
                GetTaskGroupToken,
                CancelTaskGroup,
                CloseTaskGroup,
                CloseTaskGroupWithTimeout,
                TryCloseCompletedTaskGroup,
                StartSupervisedTask,
                CloseWaitCase,
                WaitAnyWithTimeout,
                WaitAnyWithCancellation,
                CreateCancellationSource,
                GetCancellationToken,
                Cancel,
                IsCancellationRequested,
                CreateChannel,
                Send,
                TrySend,
                SendWithCancellation,
                SendWithTimeout,
                Receive,
                TryReceive,
                ReceiveWithCancellation,
                ReceiveWithTimeout,
                CloseChannel,
            ]
        ),
        "Time" => family!(
            member,
            Time,
            TimeIntrinsic,
            [TimestampMillis, MonotonicMillis, ElapsedMillis, Sleep]
        ),
        "Toml" => family!(member, Toml, TomlIntrinsic, [Parse, Stringify]),
        "Test" => resolve_test(member, first_argument),
        _ => None,
    }
}

fn resolve_http(member: &str) -> Option<Intrinsic> {
    match member {
        "Stream.ReserveState" => Some(Intrinsic::Http(HttpIntrinsic::ReserveBodyStreamState)),
        "Stream.HasState" => Some(Intrinsic::Http(HttpIntrinsic::HasBodyStreamState)),
        "Stream.LoadState" => Some(Intrinsic::Http(HttpIntrinsic::LoadBodyStreamState)),
        "Stream.StoreState" => Some(Intrinsic::Http(HttpIntrinsic::StoreBodyStreamState)),
        "Sse.ReserveState" => Some(Intrinsic::Http(HttpIntrinsic::ReserveSseDecoderState)),
        "Sse.HasState" => Some(Intrinsic::Http(HttpIntrinsic::HasSseDecoderState)),
        "Sse.LoadState" => Some(Intrinsic::Http(HttpIntrinsic::LoadSseDecoderState)),
        "Sse.StoreState" => Some(Intrinsic::Http(HttpIntrinsic::StoreSseDecoderState)),
        _ => None,
    }
}

fn resolve_console(member: &str) -> Option<Intrinsic> {
    match member {
        "WriteText" => return Some(Intrinsic::Console(ConsoleIntrinsic::Write)),
        "ReadText" => return Some(Intrinsic::Console(ConsoleIntrinsic::Read)),
        _ => {}
    }
    family!(
        member,
        Console,
        ConsoleIntrinsic,
        [
            WriteLn,
            ReadLn,
            ReadKey,
            KeyPressed,
            ReadKeyEvent,
            ClrScr,
            ClrEol,
            GotoXY,
            WhereX,
            WhereY,
            WindMin,
            WindMax,
            Window,
            TextColor,
            TextBackground,
            Delay,
            CursorOn,
            CursorOff,
            DelLine,
            InsLine,
            HighVideo,
            LowVideo,
            NormVideo,
            TextAttr,
            SetTextAttr,
            CursorBig,
            TextMode,
            LastMode,
            ScreenWidth,
            ScreenHeight,
            Sound,
            NoSound,
            AssignCrt,
            EventPending,
            ReadEvent,
            EnableRawMode,
            DisableRawMode,
            EnterAltScreen,
            LeaveAltScreen,
            EnableMouse,
            DisableMouse,
            EnableFocus,
            DisableFocus,
            EnablePaste,
            DisablePaste,
            ReadEventTimeout,
            PollEvent,
            TextColorRGB,
            TextBackgroundRGB,
            TextColor256,
            TextBackground256,
            CrtColor,
            Ansi256Color,
            RgbColor,
            BeginFrame,
            Present,
            PutCell,
            GetCell,
            FillRect,
            WriteCells,
            SaveRegion,
            RestoreRegion,
            DiscardRegion,
            DisplayWidth,
            GraphemeWidth,
            SplitGraphemes,
            AcquireInteractiveTerminal,
            ReleaseInteractiveTerminal,
        ]
    )
}

fn resolve_net(member: &str) -> Option<Intrinsic> {
    match member {
        "ReceiveBytes" => Some(Intrinsic::Net(NetIntrinsic::Read)),
        "ReceiveBytesWithCancellation" => Some(Intrinsic::Net(NetIntrinsic::ReadWithCancellation)),
        "SendBytes" => Some(Intrinsic::Net(NetIntrinsic::Write)),
        "SendBytesWithCancellation" => Some(Intrinsic::Net(NetIntrinsic::WriteWithCancellation)),
        _ => family!(
            member,
            Net,
            NetIntrinsic,
            [
                Connect,
                ConnectTls,
                ConnectWithCancellation,
                ConnectTlsWithCancellation,
                Listen,
                ListenTls,
                Accept,
                AcceptWithCancellation,
                CloseListener,
                ListenerLocalAddress,
                SetTimeout,
                Close,
            ]
        ),
    }
}

fn resolve_str(member: &str) -> Option<Intrinsic> {
    if member == "RepeatStr" {
        return Some(Intrinsic::Str(StrIntrinsic::Repeat));
    }
    family!(
        member,
        Str,
        StrIntrinsic,
        [
            Length,
            ToUpper,
            ToLower,
            Trim,
            Contains,
            StartsWith,
            EndsWith,
            Substring,
            IndexOf,
            Replace,
            Split,
            Join,
            IsNumeric,
            Repeat,
            PadLeft,
            PadRight,
            PadCenter,
            FromChar,
            CharAt,
            SetCharAt,
            Ord,
            Chr,
            Insert,
            Delete,
            Reverse,
            TrimLeft,
            TrimRight,
            LastIndexOf,
            Format,
            Map,
            Filter,
            Reduce,
        ]
    )
}

fn resolve_math(member: &str) -> Option<Intrinsic> {
    family!(
        member,
        Math,
        MathIntrinsic,
        [
            Sqrt, Pow, Floor, Ceil, Round, Sin, Cos, Log, Min, Max, Abs, Tan, ArcSin, ArcCos,
            ArcTan, ArcTan2, Exp, Log10, Log2, Trunc, Frac, Sign, Clamp,
        ]
    )
}

fn resolve_array(member: &str) -> Option<Intrinsic> {
    family!(
        member,
        Array,
        ArrayIntrinsic,
        [
            Length, Sort, Reverse, Contains, IndexOf, Slice, Map, Filter, Reduce, Concat, Fill,
            Find, FindIndex, Any, All, FlatMap, ForEach,
        ]
    )
}

fn resolve_test(member: &str, first_argument: Option<&Ty>) -> Option<Intrinsic> {
    let intrinsic = match member {
        "AssertTrue" => TestIntrinsic::AssertTrue,
        "AssertFalse" => TestIntrinsic::AssertFalse,
        "AssertEquals" => match first_argument {
            Some(Ty::Boolean) => TestIntrinsic::AssertEqualsBoolean,
            Some(Ty::String) => TestIntrinsic::AssertEqualsString,
            Some(Ty::Real) => TestIntrinsic::AssertEqualsReal,
            _ => TestIntrinsic::AssertEqualsInteger,
        },
        "Fail" => TestIntrinsic::Fail,
        "Skip" => TestIntrinsic::Skip,
        "AssertScreenLine" => TestIntrinsic::AssertScreenLine,
        "AssertScreenCell" => TestIntrinsic::AssertScreenCell,
        "PushReadLn" => TestIntrinsic::PushReadLn,
        "ScratchDir" => TestIntrinsic::ScratchDir,
        _ => return None,
    };
    Some(Intrinsic::Test(intrinsic))
}

#[cfg(test)]
mod tests;
