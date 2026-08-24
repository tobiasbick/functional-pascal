//! Canonical `Std.*` call names to stable intrinsic wire identifiers.

use fpas_bytecode::{
    ArgsIntrinsic, ArrayIntrinsic, ConsoleIntrinsic, ConvIntrinsic, DictIntrinsic, EnvIntrinsic,
    FsIntrinsic, Intrinsic, JsonIntrinsic, MathIntrinsic, NetIntrinsic, OptionIntrinsic,
    ParseIntrinsic, PathIntrinsic, ProcIntrinsic, RandomIntrinsic, ResultIntrinsic, StrIntrinsic,
    TaskIntrinsic, TestIntrinsic, TimeIntrinsic, TomlIntrinsic,
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
        "Parse" => family!(member, Parse, ParseIntrinsic, [TryInt, TryReal, TryBool]),
        "Math" => resolve_math(member),
        "Net" => family!(
            member,
            Net,
            NetIntrinsic,
            [Connect, ConnectTls, SetTimeout, Read, Write, Close]
        ),
        "Random" => family!(
            member,
            Random,
            RandomIntrinsic,
            [Random, RandomInt, Randomize]
        ),
        "Array" => resolve_array(member),
        "Dict" => family!(
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
            ]
        ),
        "Env" => family!(member, Env, EnvIntrinsic, [Get, Exists]),
        "Path" => family!(
            member,
            Path,
            PathIntrinsic,
            [Join, BaseName, DirName, Extension, Normalize]
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
                Exists,
                IsFile,
                IsDir,
                CreateDir,
                Glob,
            ]
        ),
        "Json" => family!(member, Json, JsonIntrinsic, [Parse, Stringify]),
        "Result" => family!(
            member,
            Result,
            ResultIntrinsic,
            [Unwrap, UnwrapOr, IsOk, IsError, Map, AndThen, OrElse]
        ),
        "Option" => family!(
            member,
            Option,
            OptionIntrinsic,
            [Unwrap, UnwrapOr, IsSome, IsNone, Map, AndThen, OrElse]
        ),
        "Task" => family!(member, Task, TaskIntrinsic, [Wait, WaitAll]),
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

fn resolve_console(member: &str) -> Option<Intrinsic> {
    family!(
        member,
        Console,
        ConsoleIntrinsic,
        [
            Write,
            WriteLn,
            ReadLn,
            Read,
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
        _ => return None,
    };
    Some(Intrinsic::Test(intrinsic))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stable_intrinsic_id_has_a_canonical_source_call() {
        for intrinsic in Intrinsic::all() {
            let (name, argument_type) = canonical_test_call(intrinsic);
            assert_eq!(
                resolve(&name, argument_type.as_ref()),
                Some(intrinsic),
                "canonical catalog did not resolve {intrinsic:?} through {name}"
            );
        }
    }

    fn canonical_test_call(intrinsic: Intrinsic) -> (String, Option<Ty>) {
        match intrinsic {
            Intrinsic::Str(StrIntrinsic::Repeat) => ("Std.Str.RepeatStr".into(), None),
            Intrinsic::Test(TestIntrinsic::AssertEqualsInteger) => {
                ("Std.Test.AssertEquals".into(), Some(Ty::Integer))
            }
            Intrinsic::Test(TestIntrinsic::AssertEqualsBoolean) => {
                ("Std.Test.AssertEquals".into(), Some(Ty::Boolean))
            }
            Intrinsic::Test(TestIntrinsic::AssertEqualsString) => {
                ("Std.Test.AssertEquals".into(), Some(Ty::String))
            }
            Intrinsic::Test(TestIntrinsic::AssertEqualsReal) => {
                ("Std.Test.AssertEquals".into(), Some(Ty::Real))
            }
            intrinsic => {
                let debug = format!("{intrinsic:?}");
                let (family, member) = debug
                    .split_once('(')
                    .expect("intrinsic debug form contains family and member");
                (
                    format!("Std.{family}.{}", member.trim_end_matches(')')),
                    None,
                )
            }
        }
    }
}
