use super::*;

/// All intrinsic variants — used by tests to verify completeness of `from_u16` coverage.
const ALL_INTRINSICS: &[Intrinsic] = &[
    Intrinsic::Args(ArgsIntrinsic::ParamCount),
    Intrinsic::Args(ArgsIntrinsic::ParamStr),
    Intrinsic::Env(EnvIntrinsic::Get),
    Intrinsic::Env(EnvIntrinsic::Exists),
    Intrinsic::Path(PathIntrinsic::Join),
    Intrinsic::Path(PathIntrinsic::BaseName),
    Intrinsic::Path(PathIntrinsic::DirName),
    Intrinsic::Path(PathIntrinsic::Extension),
    Intrinsic::Path(PathIntrinsic::Normalize),
    Intrinsic::Proc(ProcIntrinsic::Run),
    Intrinsic::Fs(FsIntrinsic::ReadText),
    Intrinsic::Fs(FsIntrinsic::WriteText),
    Intrinsic::Fs(FsIntrinsic::Exists),
    Intrinsic::Fs(FsIntrinsic::IsFile),
    Intrinsic::Fs(FsIntrinsic::IsDir),
    Intrinsic::Fs(FsIntrinsic::CreateDir),
    Intrinsic::Time(TimeIntrinsic::TimestampMillis),
    Intrinsic::Time(TimeIntrinsic::MonotonicMillis),
    Intrinsic::Time(TimeIntrinsic::ElapsedMillis),
    Intrinsic::Time(TimeIntrinsic::Sleep),
    Intrinsic::Console(ConsoleIntrinsic::ReadLn),
    Intrinsic::Console(ConsoleIntrinsic::Read),
    Intrinsic::Console(ConsoleIntrinsic::ReadKey),
    Intrinsic::Console(ConsoleIntrinsic::KeyPressed),
    Intrinsic::Console(ConsoleIntrinsic::ReadKeyEvent),
    Intrinsic::Console(ConsoleIntrinsic::ClrScr),
    Intrinsic::Console(ConsoleIntrinsic::ClrEol),
    Intrinsic::Console(ConsoleIntrinsic::GotoXY),
    Intrinsic::Console(ConsoleIntrinsic::WhereX),
    Intrinsic::Console(ConsoleIntrinsic::WhereY),
    Intrinsic::Console(ConsoleIntrinsic::WindMin),
    Intrinsic::Console(ConsoleIntrinsic::WindMax),
    Intrinsic::Console(ConsoleIntrinsic::Window),
    Intrinsic::Console(ConsoleIntrinsic::TextColor),
    Intrinsic::Console(ConsoleIntrinsic::TextBackground),
    Intrinsic::Console(ConsoleIntrinsic::Delay),
    Intrinsic::Console(ConsoleIntrinsic::CursorOn),
    Intrinsic::Console(ConsoleIntrinsic::CursorOff),
    Intrinsic::Console(ConsoleIntrinsic::DelLine),
    Intrinsic::Console(ConsoleIntrinsic::InsLine),
    Intrinsic::Console(ConsoleIntrinsic::HighVideo),
    Intrinsic::Console(ConsoleIntrinsic::LowVideo),
    Intrinsic::Console(ConsoleIntrinsic::NormVideo),
    Intrinsic::Console(ConsoleIntrinsic::TextAttr),
    Intrinsic::Console(ConsoleIntrinsic::SetTextAttr),
    Intrinsic::Console(ConsoleIntrinsic::CursorBig),
    Intrinsic::Console(ConsoleIntrinsic::TextMode),
    Intrinsic::Console(ConsoleIntrinsic::LastMode),
    Intrinsic::Console(ConsoleIntrinsic::ScreenWidth),
    Intrinsic::Console(ConsoleIntrinsic::ScreenHeight),
    Intrinsic::Console(ConsoleIntrinsic::Sound),
    Intrinsic::Console(ConsoleIntrinsic::NoSound),
    Intrinsic::Console(ConsoleIntrinsic::AssignCrt),
    Intrinsic::Console(ConsoleIntrinsic::EventPending),
    Intrinsic::Console(ConsoleIntrinsic::ReadEvent),
    Intrinsic::Console(ConsoleIntrinsic::EnableRawMode),
    Intrinsic::Console(ConsoleIntrinsic::DisableRawMode),
    Intrinsic::Console(ConsoleIntrinsic::EnterAltScreen),
    Intrinsic::Console(ConsoleIntrinsic::LeaveAltScreen),
    Intrinsic::Console(ConsoleIntrinsic::EnableMouse),
    Intrinsic::Console(ConsoleIntrinsic::DisableMouse),
    Intrinsic::Console(ConsoleIntrinsic::EnableFocus),
    Intrinsic::Console(ConsoleIntrinsic::DisableFocus),
    Intrinsic::Console(ConsoleIntrinsic::EnablePaste),
    Intrinsic::Console(ConsoleIntrinsic::DisablePaste),
    Intrinsic::Console(ConsoleIntrinsic::ReadEventTimeout),
    Intrinsic::Console(ConsoleIntrinsic::PollEvent),
    Intrinsic::Console(ConsoleIntrinsic::TextColorRGB),
    Intrinsic::Console(ConsoleIntrinsic::TextBackgroundRGB),
    Intrinsic::Console(ConsoleIntrinsic::TextColor256),
    Intrinsic::Console(ConsoleIntrinsic::TextBackground256),
    Intrinsic::Str(StrIntrinsic::Length),
    Intrinsic::Str(StrIntrinsic::ToUpper),
    Intrinsic::Str(StrIntrinsic::ToLower),
    Intrinsic::Str(StrIntrinsic::Trim),
    Intrinsic::Str(StrIntrinsic::Contains),
    Intrinsic::Str(StrIntrinsic::StartsWith),
    Intrinsic::Str(StrIntrinsic::EndsWith),
    Intrinsic::Str(StrIntrinsic::Substring),
    Intrinsic::Str(StrIntrinsic::IndexOf),
    Intrinsic::Str(StrIntrinsic::Replace),
    Intrinsic::Str(StrIntrinsic::Split),
    Intrinsic::Str(StrIntrinsic::Join),
    Intrinsic::Str(StrIntrinsic::IsNumeric),
    Intrinsic::Str(StrIntrinsic::Repeat),
    Intrinsic::Str(StrIntrinsic::PadLeft),
    Intrinsic::Str(StrIntrinsic::PadRight),
    Intrinsic::Str(StrIntrinsic::PadCenter),
    Intrinsic::Str(StrIntrinsic::FromChar),
    Intrinsic::Str(StrIntrinsic::CharAt),
    Intrinsic::Str(StrIntrinsic::SetCharAt),
    Intrinsic::Str(StrIntrinsic::Ord),
    Intrinsic::Str(StrIntrinsic::Chr),
    Intrinsic::Str(StrIntrinsic::Insert),
    Intrinsic::Str(StrIntrinsic::Delete),
    Intrinsic::Str(StrIntrinsic::Reverse),
    Intrinsic::Str(StrIntrinsic::TrimLeft),
    Intrinsic::Str(StrIntrinsic::TrimRight),
    Intrinsic::Str(StrIntrinsic::LastIndexOf),
    Intrinsic::Str(StrIntrinsic::Format),
    Intrinsic::Conv(ConvIntrinsic::IntToStr),
    Intrinsic::Conv(ConvIntrinsic::StrToInt),
    Intrinsic::Conv(ConvIntrinsic::RealToStr),
    Intrinsic::Conv(ConvIntrinsic::StrToReal),
    Intrinsic::Conv(ConvIntrinsic::IntToReal),
    Intrinsic::Conv(ConvIntrinsic::BoolToStr),
    Intrinsic::Conv(ConvIntrinsic::StrToBool),
    Intrinsic::Conv(ConvIntrinsic::IntToHex),
    Intrinsic::Conv(ConvIntrinsic::HexToInt),
    Intrinsic::Parse(ParseIntrinsic::TryInt),
    Intrinsic::Parse(ParseIntrinsic::TryReal),
    Intrinsic::Parse(ParseIntrinsic::TryBool),
    Intrinsic::Math(MathIntrinsic::Sqrt),
    Intrinsic::Math(MathIntrinsic::Pow),
    Intrinsic::Math(MathIntrinsic::Floor),
    Intrinsic::Math(MathIntrinsic::Ceil),
    Intrinsic::Math(MathIntrinsic::Round),
    Intrinsic::Math(MathIntrinsic::Sin),
    Intrinsic::Math(MathIntrinsic::Cos),
    Intrinsic::Math(MathIntrinsic::Log),
    Intrinsic::Math(MathIntrinsic::Min),
    Intrinsic::Math(MathIntrinsic::Max),
    Intrinsic::Math(MathIntrinsic::Abs),
    Intrinsic::Math(MathIntrinsic::Tan),
    Intrinsic::Math(MathIntrinsic::ArcSin),
    Intrinsic::Math(MathIntrinsic::ArcCos),
    Intrinsic::Math(MathIntrinsic::ArcTan),
    Intrinsic::Math(MathIntrinsic::ArcTan2),
    Intrinsic::Math(MathIntrinsic::Exp),
    Intrinsic::Math(MathIntrinsic::Log10),
    Intrinsic::Math(MathIntrinsic::Log2),
    Intrinsic::Math(MathIntrinsic::Trunc),
    Intrinsic::Math(MathIntrinsic::Frac),
    Intrinsic::Math(MathIntrinsic::Sign),
    Intrinsic::Math(MathIntrinsic::Clamp),
    Intrinsic::Random(RandomIntrinsic::Random),
    Intrinsic::Random(RandomIntrinsic::RandomInt),
    Intrinsic::Random(RandomIntrinsic::Randomize),
    Intrinsic::Array(ArrayIntrinsic::Length),
    Intrinsic::Array(ArrayIntrinsic::Sort),
    Intrinsic::Array(ArrayIntrinsic::Reverse),
    Intrinsic::Array(ArrayIntrinsic::Contains),
    Intrinsic::Array(ArrayIntrinsic::IndexOf),
    Intrinsic::Array(ArrayIntrinsic::Slice),
    Intrinsic::Array(ArrayIntrinsic::Map),
    Intrinsic::Array(ArrayIntrinsic::Filter),
    Intrinsic::Array(ArrayIntrinsic::Reduce),
    Intrinsic::Array(ArrayIntrinsic::Concat),
    Intrinsic::Array(ArrayIntrinsic::Fill),
    Intrinsic::Array(ArrayIntrinsic::Find),
    Intrinsic::Array(ArrayIntrinsic::FindIndex),
    Intrinsic::Array(ArrayIntrinsic::Any),
    Intrinsic::Array(ArrayIntrinsic::All),
    Intrinsic::Array(ArrayIntrinsic::FlatMap),
    Intrinsic::Array(ArrayIntrinsic::ForEach),
    Intrinsic::Result(ResultIntrinsic::Unwrap),
    Intrinsic::Result(ResultIntrinsic::UnwrapOr),
    Intrinsic::Result(ResultIntrinsic::IsOk),
    Intrinsic::Result(ResultIntrinsic::IsError),
    Intrinsic::Result(ResultIntrinsic::Map),
    Intrinsic::Result(ResultIntrinsic::AndThen),
    Intrinsic::Result(ResultIntrinsic::OrElse),
    Intrinsic::Option(OptionIntrinsic::Unwrap),
    Intrinsic::Option(OptionIntrinsic::UnwrapOr),
    Intrinsic::Option(OptionIntrinsic::IsSome),
    Intrinsic::Option(OptionIntrinsic::IsNone),
    Intrinsic::Option(OptionIntrinsic::Map),
    Intrinsic::Option(OptionIntrinsic::AndThen),
    Intrinsic::Option(OptionIntrinsic::OrElse),
    Intrinsic::Task(TaskIntrinsic::Wait),
    Intrinsic::Task(TaskIntrinsic::WaitAll),
    Intrinsic::Dict(DictIntrinsic::Length),
    Intrinsic::Dict(DictIntrinsic::ContainsKey),
    Intrinsic::Dict(DictIntrinsic::Keys),
    Intrinsic::Dict(DictIntrinsic::Values),
    Intrinsic::Dict(DictIntrinsic::Remove),
    Intrinsic::Dict(DictIntrinsic::Get),
    Intrinsic::Dict(DictIntrinsic::Merge),
    Intrinsic::Dict(DictIntrinsic::Map),
    Intrinsic::Dict(DictIntrinsic::Filter),
    Intrinsic::Graph(GraphIntrinsic::ApplicationOpen),
    Intrinsic::Graph(GraphIntrinsic::ApplicationClose),
    Intrinsic::Graph(GraphIntrinsic::ApplicationSize),
    Intrinsic::Graph(GraphIntrinsic::ApplicationRequestRedraw),
    Intrinsic::Graph(GraphIntrinsic::ApplicationConfigure),
    Intrinsic::Graph(GraphIntrinsic::ApplicationRun),
    Intrinsic::Graph(GraphIntrinsic::ApplicationUploadFrame),
    Intrinsic::Graph(GraphIntrinsic::ApplicationClear),
    Intrinsic::Graph(GraphIntrinsic::ApplicationPutPixel),
    Intrinsic::Graph(GraphIntrinsic::ApplicationPresent),
    Intrinsic::Graph(GraphIntrinsic::ApplicationDrawLine),
    Intrinsic::Graph(GraphIntrinsic::ApplicationDrawRect),
    Intrinsic::Graph(GraphIntrinsic::ApplicationFillRect),
    Intrinsic::Graph(GraphIntrinsic::ApplicationDrawCircle),
    Intrinsic::Graph(GraphIntrinsic::ApplicationDrawText),
    Intrinsic::Graph(GraphIntrinsic::HostRequestQuit),
    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnKeyPressed),
    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnResize),
    Intrinsic::Graph(GraphIntrinsic::HostProcessNext),
    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnPaint),
    Intrinsic::Graph(GraphIntrinsic::HostDispatchRedraw),
    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnIdle),
    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnExit),
    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnMouse),
    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnWheel),
    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnCloseRequested),
    Intrinsic::Graph(GraphIntrinsic::OpenForTest),
    Intrinsic::Graph(GraphIntrinsic::TestSendKey),
    Intrinsic::Json(JsonIntrinsic::Parse),
    Intrinsic::Json(JsonIntrinsic::Stringify),
    Intrinsic::Tui(TuiIntrinsic::ApplicationOpen),
    Intrinsic::Tui(TuiIntrinsic::ApplicationClose),
    Intrinsic::Tui(TuiIntrinsic::ApplicationSize),
    Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw),
    Intrinsic::Tui(TuiIntrinsic::ApplicationConfigure),
    Intrinsic::Tui(TuiIntrinsic::ApplicationRun),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnKeyPressed),
    Intrinsic::Tui(TuiIntrinsic::HostInvokeOnKeyPressed),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnResize),
    Intrinsic::Tui(TuiIntrinsic::HostProcessNext),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaint),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnIdle),
    Intrinsic::Tui(TuiIntrinsic::HostDispatchRedraw),
    Intrinsic::Tui(TuiIntrinsic::HostRunLoop),
    Intrinsic::Tui(TuiIntrinsic::HostRequestQuit),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnExit),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaste),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusGained),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusLost),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnActivate),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnDeactivate),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnCommand),
    Intrinsic::Tui(TuiIntrinsic::HostBindCommand),
    Intrinsic::Tui(TuiIntrinsic::HostEnterModal),
    Intrinsic::Tui(TuiIntrinsic::HostLeaveModal),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterView),
    Intrinsic::Tui(TuiIntrinsic::HostUnregisterView),
    Intrinsic::Tui(TuiIntrinsic::HostPushChildView),
    Intrinsic::Tui(TuiIntrinsic::HostAttachViewToActiveModal),
    Intrinsic::Tui(TuiIntrinsic::HostSetViewRect),
    Intrinsic::Tui(TuiIntrinsic::HostSetViewParent),
    Intrinsic::Tui(TuiIntrinsic::HostSetViewVisible),
    Intrinsic::Tui(TuiIntrinsic::HostSetViewEnabled),
    Intrinsic::Tui(TuiIntrinsic::CreateDialog),
    Intrinsic::Tui(TuiIntrinsic::CreateButton),
    Intrinsic::Tui(TuiIntrinsic::AddChild),
    Intrinsic::Tui(TuiIntrinsic::RegisterOnCommand),
    Intrinsic::Tui(TuiIntrinsic::Pump),
    Intrinsic::Tui(TuiIntrinsic::Quit),
    Intrinsic::Tui(TuiIntrinsic::TestClickButton),
    Intrinsic::Tui(TuiIntrinsic::HostSetViewLayout),
    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnViewPaint),
    Intrinsic::Tui(TuiIntrinsic::ApplicationShowModal),
    Intrinsic::Tui(TuiIntrinsic::ApplicationCloseModal),
    Intrinsic::Tui(TuiIntrinsic::HostBindCommandToView),
    Intrinsic::Tui(TuiIntrinsic::HostBindCommandToActiveModal),
    Intrinsic::Tui(TuiIntrinsic::ApplicationShowDialog),
    Intrinsic::Tui(TuiIntrinsic::HostSetActiveModalResult),
    Intrinsic::Tui(TuiIntrinsic::HostCreateSolidFillView),
    Intrinsic::Tui(TuiIntrinsic::HostCreateStatusBarView),
    Intrinsic::Tui(TuiIntrinsic::HostSetStatusBarSegments),
    Intrinsic::Tui(TuiIntrinsic::OpenForTest),
    Intrinsic::Tui(TuiIntrinsic::TestPump),
    Intrinsic::Tui(TuiIntrinsic::TestPumpUntilIdle),
    Intrinsic::Tui(TuiIntrinsic::CloseForTest),
    Intrinsic::Tui(TuiIntrinsic::TestSendKey),
    Intrinsic::Tui(TuiIntrinsic::TestSendMouse),
    Intrinsic::Tui(TuiIntrinsic::TestMoveMouse),
    Intrinsic::Tui(TuiIntrinsic::TestClickMouse),
    Intrinsic::Tui(TuiIntrinsic::TestResize),
    Intrinsic::Tui(TuiIntrinsic::TestPaste),
    Intrinsic::Tui(TuiIntrinsic::TestFocus),
    Intrinsic::Tui(TuiIntrinsic::QueryScreenSize),
    Intrinsic::Tui(TuiIntrinsic::QueryScreenLine),
    Intrinsic::Tui(TuiIntrinsic::QueryScreenCell),
    Intrinsic::Test(TestIntrinsic::AssertTrue),
    Intrinsic::Test(TestIntrinsic::AssertFalse),
    Intrinsic::Test(TestIntrinsic::AssertEqualsInteger),
    Intrinsic::Test(TestIntrinsic::Fail),
    Intrinsic::Test(TestIntrinsic::Skip),
    Intrinsic::Test(TestIntrinsic::AssertEqualsBoolean),
    Intrinsic::Test(TestIntrinsic::AssertEqualsString),
    Intrinsic::Test(TestIntrinsic::AssertEqualsReal),
    Intrinsic::Test(TestIntrinsic::AssertScreenLine),
    Intrinsic::Test(TestIntrinsic::AssertScreenCell),
    Intrinsic::Test(TestIntrinsic::AssertViewRect),
    Intrinsic::Test(TestIntrinsic::PushReadLn),
];

#[test]
fn intrinsic_round_trip_encode_decode() {
    for &intr in ALL_INTRINSICS {
        let encoded: u16 = intr.into();
        let decoded = Intrinsic::from_u16(encoded);
        assert_eq!(
            decoded,
            Some(intr),
            "round-trip failed for {intr:?} (discriminant {encoded}): from_u16 returned {decoded:?}"
        );
    }
}

#[test]
fn all_intrinsics_list_is_complete() {
    let count_in_list = ALL_INTRINSICS.len();
    let mut found_via_probe = 0usize;
    for raw in 0..=u16::MAX {
        if Intrinsic::from_u16(raw).is_some() {
            found_via_probe += 1;
        }
    }
    assert_eq!(
        count_in_list, found_via_probe,
        "ALL_INTRINSICS has {count_in_list} entries but from_u16 recognises {found_via_probe} — \
         a variant was added without updating ALL_INTRINSICS"
    );
}

#[test]
fn intrinsic_wire_values_are_globally_unique() -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    let intrinsic_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/intrinsic");
    let mut by_value: HashMap<u16, Vec<String>> = HashMap::new();

    fn scan_dir(
        dir: &std::path::Path,
        by_value: &mut HashMap<u16, Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, by_value)?;
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if matches!(file_name, "mod.rs" | "tests.rs") {
                continue;
            }
            let source = fs::read_to_string(&path)?;
            let parent = dir
                .parent()
                .ok_or_else(|| format!("{} has no parent", dir.display()))?;
            let label = path.strip_prefix(parent)?.display().to_string();
            for line in source.lines() {
                let Some((lhs, rhs)) = line.split_once('=') else {
                    continue;
                };
                if !rhs.trim_end().ends_with(',') {
                    continue;
                }
                let Some(value_text) = rhs.trim().strip_suffix(',') else {
                    continue;
                };
                let Ok(value) = value_text.trim().parse::<u16>() else {
                    continue;
                };
                let variant = lhs.rsplit("::").next().unwrap_or(lhs).trim();
                if variant.is_empty() || variant.starts_with("//") {
                    continue;
                }
                by_value
                    .entry(value)
                    .or_default()
                    .push(format!("{label}:{variant}"));
            }
        }
        Ok(())
    }

    scan_dir(&intrinsic_root, &mut by_value)?;

    let duplicates: Vec<_> = by_value
        .iter()
        .filter(|(_, owners)| owners.len() > 1)
        .collect();
    assert!(
        duplicates.is_empty(),
        "duplicate intrinsic wire values: {duplicates:?}"
    );
    Ok(())
}
