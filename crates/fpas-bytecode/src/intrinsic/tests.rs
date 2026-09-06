use super::*;

/// All intrinsic variants — used by tests to verify completeness of `from_u16` coverage.
const ALL_INTRINSICS: &[Intrinsic] = &[
    Intrinsic::Console(ConsoleIntrinsic::Write),
    Intrinsic::Console(ConsoleIntrinsic::WriteLn),
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
    Intrinsic::Proc(ProcIntrinsic::CurrentExecutable),
    Intrinsic::Proc(ProcIntrinsic::RunCapture),
    Intrinsic::Fs(FsIntrinsic::ReadText),
    Intrinsic::Fs(FsIntrinsic::WriteText),
    Intrinsic::Fs(FsIntrinsic::WriteTextAtomic),
    Intrinsic::Fs(FsIntrinsic::Exists),
    Intrinsic::Fs(FsIntrinsic::IsFile),
    Intrinsic::Fs(FsIntrinsic::IsDir),
    Intrinsic::Fs(FsIntrinsic::CreateDir),
    Intrinsic::Fs(FsIntrinsic::Glob),
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
    Intrinsic::Console(ConsoleIntrinsic::CrtColor),
    Intrinsic::Console(ConsoleIntrinsic::Ansi256Color),
    Intrinsic::Console(ConsoleIntrinsic::RgbColor),
    Intrinsic::Console(ConsoleIntrinsic::BeginFrame),
    Intrinsic::Console(ConsoleIntrinsic::Present),
    Intrinsic::Console(ConsoleIntrinsic::PutCell),
    Intrinsic::Console(ConsoleIntrinsic::GetCell),
    Intrinsic::Console(ConsoleIntrinsic::FillRect),
    Intrinsic::Console(ConsoleIntrinsic::WriteCells),
    Intrinsic::Console(ConsoleIntrinsic::SaveRegion),
    Intrinsic::Console(ConsoleIntrinsic::RestoreRegion),
    Intrinsic::Console(ConsoleIntrinsic::DiscardRegion),
    Intrinsic::Console(ConsoleIntrinsic::DisplayWidth),
    Intrinsic::Console(ConsoleIntrinsic::GraphemeWidth),
    Intrinsic::Console(ConsoleIntrinsic::SplitGraphemes),
    Intrinsic::Console(ConsoleIntrinsic::AcquireInteractiveTerminal),
    Intrinsic::Console(ConsoleIntrinsic::ReleaseInteractiveTerminal),
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
    Intrinsic::Net(NetIntrinsic::Connect),
    Intrinsic::Net(NetIntrinsic::SetTimeout),
    Intrinsic::Net(NetIntrinsic::Read),
    Intrinsic::Net(NetIntrinsic::Write),
    Intrinsic::Net(NetIntrinsic::Close),
    Intrinsic::Net(NetIntrinsic::ConnectTls),
    Intrinsic::Net(NetIntrinsic::Listen),
    Intrinsic::Net(NetIntrinsic::Accept),
    Intrinsic::Net(NetIntrinsic::CloseListener),
    Intrinsic::Net(NetIntrinsic::ListenTls),
    Intrinsic::Net(NetIntrinsic::AcceptWithCancellation),
    Intrinsic::Net(NetIntrinsic::ReadWithCancellation),
    Intrinsic::Net(NetIntrinsic::WriteWithCancellation),
    Intrinsic::Net(NetIntrinsic::ConnectWithCancellation),
    Intrinsic::Net(NetIntrinsic::ConnectTlsWithCancellation),
    Intrinsic::Http(HttpIntrinsic::ReserveBodyStreamState),
    Intrinsic::Http(HttpIntrinsic::HasBodyStreamState),
    Intrinsic::Http(HttpIntrinsic::LoadBodyStreamState),
    Intrinsic::Http(HttpIntrinsic::StoreBodyStreamState),
    Intrinsic::Http(HttpIntrinsic::ReserveSseDecoderState),
    Intrinsic::Http(HttpIntrinsic::HasSseDecoderState),
    Intrinsic::Http(HttpIntrinsic::LoadSseDecoderState),
    Intrinsic::Http(HttpIntrinsic::StoreSseDecoderState),
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
    Intrinsic::Task(TaskIntrinsic::WaitAny),
    Intrinsic::Task(TaskIntrinsic::WaitAnyWithTimeout),
    Intrinsic::Task(TaskIntrinsic::WaitAnyWithCancellation),
    Intrinsic::Task(TaskIntrinsic::CreateCancellationSource),
    Intrinsic::Task(TaskIntrinsic::GetCancellationToken),
    Intrinsic::Task(TaskIntrinsic::Cancel),
    Intrinsic::Task(TaskIntrinsic::IsCancellationRequested),
    Intrinsic::Task(TaskIntrinsic::CreateChannel),
    Intrinsic::Task(TaskIntrinsic::Send),
    Intrinsic::Task(TaskIntrinsic::TrySend),
    Intrinsic::Task(TaskIntrinsic::SendWithCancellation),
    Intrinsic::Task(TaskIntrinsic::SendWithTimeout),
    Intrinsic::Task(TaskIntrinsic::Receive),
    Intrinsic::Task(TaskIntrinsic::TryReceive),
    Intrinsic::Task(TaskIntrinsic::ReceiveWithCancellation),
    Intrinsic::Task(TaskIntrinsic::ReceiveWithTimeout),
    Intrinsic::Task(TaskIntrinsic::CloseChannel),
    Intrinsic::Dict(DictIntrinsic::Length),
    Intrinsic::Dict(DictIntrinsic::ContainsKey),
    Intrinsic::Dict(DictIntrinsic::Keys),
    Intrinsic::Dict(DictIntrinsic::Values),
    Intrinsic::Dict(DictIntrinsic::Remove),
    Intrinsic::Dict(DictIntrinsic::Get),
    Intrinsic::Dict(DictIntrinsic::Merge),
    Intrinsic::Dict(DictIntrinsic::Map),
    Intrinsic::Dict(DictIntrinsic::Filter),
    Intrinsic::Json(JsonIntrinsic::Parse),
    Intrinsic::Json(JsonIntrinsic::Stringify),
    Intrinsic::Toml(TomlIntrinsic::Parse),
    Intrinsic::Toml(TomlIntrinsic::Stringify),
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
    Intrinsic::Test(TestIntrinsic::PushReadLn),
    Intrinsic::Test(TestIntrinsic::ScratchDir),
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
