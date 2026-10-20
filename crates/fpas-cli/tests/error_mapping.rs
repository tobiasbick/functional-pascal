//! Constructor values and MapError across source, cached units and runtime routes.
#![allow(
    clippy::expect_used,
    reason = "integration fixtures assert process and file outcomes"
)]

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "fpas-map-error-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("fixture directory");
        fs::write(root.join("main.fpas"), source).expect("fixture source");
        Self(root)
    }
    fn run(&self, args: &[&str], code: i32) -> Output {
        let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .expect("CLI process");
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("fixture cleanup");
    }
}

#[test]
fn imported_generic_constructor_values_survive_cached_units_and_program_images() {
    let f = Fixture::new(
        r#"program Main;
uses App.Errors as Imported, Std.Test as Test;
type Alias = Imported.Leaf;
begin
  const MakePair: function(Word: string; Count: integer): Imported.Entry := Imported.Entry.Pair;
  Test.AssertTrue(MakePair(Count := 2, Word := 'enum') is Imported.Entry.Pair('enum', 2));
  const MakeBox: function(Text: string): Imported.Box of string := Imported.Box.Item;
  Test.AssertTrue(MakeBox('boxed') is Imported.Box.Item('boxed'));
  const Input: result of (integer, Alias) := Error(Alias.Bad('first'));
  const Wrapped := Input.MapError(Imported.Context.Failed);
  Test.AssertTrue(Wrapped is Error(Imported.Context.Failed(Alias.Bad('first'))));
  const Other: result of (boolean, Imported.Other) := Error(Imported.Other.Bad(2));
  const Second := Imported.Map(Other);
  Test.AssertTrue(Second is Error(Imported.Context.Failed(Imported.Other.Bad(2))));
  const Factory: function(Value: Imported.Leaf): Imported.Context of Imported.Leaf := Imported.Context.Failed;
  case Imported.Map(Input) of
    when Ok(_): Test.AssertTrue(false);
    when Error(const Value): Test.AssertEquals(Factory(Alias.Bad('first')).Message(), Value.Message());
  end case;
end."#,
    );
    fs::write(
        f.0.join("errors.fpas"),
        r#"unit App.Errors;
public type Leaf = error Bad(Text: string); end error;
public type Other = error Bad(Code: integer); end error;
public type Context of E: Error = error Failed(Cause: E); end error;
public type Entry = enum Pair(Text: string; Count: integer); end enum;
public type Box of T = enum Item(Value: T); end enum;
public function Map<T, E: Error>(Value: result of (T, E)): result of (T, Context of E);
begin return Value.MapError(Context.Failed); end function;
end unit;"#,
    )
    .expect("unit");
    fs::write(f.0.join("app.fpasprj"), "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n").expect("manifest");
    f.run(&["run", "app.fpasprj"], 0);
    assert!(f.0.join("errors.fpascu").is_file());
    let cached = fs::read(f.0.join("errors.fpascu")).expect("cached unit");
    f.run(&["run", "app.fpasprj"], 0);
    assert_eq!(
        cached,
        fs::read(f.0.join("errors.fpascu")).expect("cached unit")
    );
    f.run(&["build", "app.fpasprj"], 0);
    f.run(&["run", "app.fpascp"], 0);
}

#[test]
fn map_error_resumes_nested_callbacks_and_preserves_trace_bounds() {
    let f = Fixture::new(
        r#"program Scheduled;
uses Std.Tasks as Tasks, Std.Time as Time, Std.Test as Test;
var Calls: integer := 0;
function Inner(Value: string): integer;
begin Time.Sleep(1); return Value.Length(); end function;
function Outer(Value: string): integer;
begin
  const Input: result of (boolean, string) := Error(Value);
  return case Input.MapError(Inner) of when Ok(_): 0; when Error(const N): N; end case;
end function;
function Work(): integer;
begin
  const Input: result of (boolean, string) := Error('abc');
  return case Input.MapError(Outer) of when Ok(_): 0; when Error(const N): N; end case;
end function;
begin scope
  const Pending := go Work();
  Calls := Tasks.Wait(Pending);
  Test.AssertEquals(3, Calls);
end scope; end."#,
    );
    f.run(&["run", "main.fpas"], 0);
    let output = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Calls",
            "--schedule-seed",
            "42",
        ],
        0,
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("debug JSON");
    assert_eq!(report["trace"]["complete"], true);
    assert_eq!(report["trace"]["event_count"], 2);
    assert_eq!(report["schedule"]["status"], "controlled");
    let output = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Calls",
            "--trace-limit",
            "1",
        ],
        3,
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("debug JSON");
    assert_eq!(report["trace"]["complete"], false);
}

#[test]
fn mapper_panics_and_contracts_remain_runtime_failures_on_main_and_tasks() {
    for failure in ["panic('mapper failed');", "return 1;"] {
        for scheduled in [false, true] {
            let contract = if failure.starts_with("return") {
                "requires Value = 'allowed';"
            } else {
                ""
            };
            let invoke = if scheduled {
                "scope const Pending := go Work(); discard Tasks.Wait(Pending); end scope;"
            } else {
                "discard Work();"
            };
            let f = Fixture::new(&format!(
                "program Failure; uses Std.Tasks as Tasks; function Mapper(Value: string): integer; {contract} begin {failure} end function; function Work(): integer; begin const Input: result of (integer, string) := Error('bad'); discard Input.MapError(Mapper); return 0; end function; begin {invoke} end."
            ));
            let output = f.run(&["run", "main.fpas", "--failure-dump", "failure.json"], 2);
            assert!(!output.stderr.is_empty());
            let report: serde_json::Value =
                serde_json::from_slice(&fs::read(f.0.join("failure.json")).expect("failure dump"))
                    .expect("failure JSON");
            assert!(report.is_object());
        }
    }
}

#[test]
fn synchronous_mapper_mutations_obey_callback_trace_bounds() {
    let f = Fixture::new(
        r#"program TracedMapper;
var Calls: integer := 0;
function Mapper(Value: string): integer;
begin Calls := Calls + 1; return Value.Length(); end function;
begin
  const Input: result of (boolean, string) := Error('abc');
  discard Input.MapError(Mapper);
end."#,
    );
    let output = f.run(&["debug", "main.fpas", "--trace", "Calls"], 0);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("debug JSON");
    assert_eq!(report["trace"]["complete"], true);
    assert_eq!(report["trace"]["event_count"], 2);
    assert_eq!(report["trace"]["events"][1]["new_value"]["scalar"], "1");
    let output = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Calls",
            "--trace-limit",
            "1",
        ],
        3,
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("debug JSON");
    assert_eq!(report["outcome"], "limit");
    assert_eq!(report["trace"]["complete"], false);
    assert!(
        report["trace"]["reasons"]
            .as_array()
            .expect("trace reasons")
            .contains(&serde_json::Value::from("trace_event_limit"))
    );
}

#[test]
fn host_and_parser_families_preserve_structural_failures_in_program_images() {
    let f = Fixture::new(
        r#"program HostErrors;
uses Std.Fs as Fs, Std.Proc as Proc, Std.Parse as Parse, Std.Json as Json,
  Std.Toml as Toml, Std.Json.Fields as JsonFields, Std.Toml.Fields as TomlFields,
  Std.Crypto as Crypto, Std.Errors as Errors, Std.Test as Test;
begin
  Test.AssertTrue(Fs.ReadText('missing') is Error(Fs.FsError.FileNotFound('missing', Errors.HostError.Failure('Std.Fs.ReadText', _, Some(_), Some(_), _))));
  Test.AssertTrue(Fs.ReadText('invalid.txt') is Error(Fs.FsError.InvalidEncoding('invalid.txt', _)));
  Test.AssertTrue(Fs.ReadText('large.txt') is Error(Fs.FsError.LimitExceeded('Std.Fs.ReadText', 67108864)));
  Test.AssertTrue(Fs.ReadText('bad'#0) is Error(Fs.FsError.InvalidArgument('Path', _)));
  Test.AssertTrue(Fs.CopyFile('invalid.txt', 'bad'#0) is Error(Fs.FsError.InvalidArgument('Target', _)));
  Test.AssertTrue(Fs.Glob('[') is Error(Fs.FsError.InvalidPattern('[', _)));
  Test.AssertTrue(Fs.FileSize('directory') is Error(Fs.FsError.InvalidArgument('Path', _)));
  Test.AssertTrue(Fs.CreateDir('directory') is Error(Fs.FsError.AlreadyExists('directory', _)));
  Test.AssertTrue(Fs.CopyDirAll('invalid.txt', 'target') is Error(Fs.FsError.InvalidArgument('Source', _)));
  Test.AssertTrue(Fs.CopyDirAll('directory', 'invalid.txt') is Error(Fs.FsError.AlreadyExists('invalid.txt', _)));
  Test.AssertTrue(Fs.DeleteDir('directory') is Error(Fs.FsError.NotEmpty('directory', _)));
  Test.AssertTrue(Fs.ReadDir('invalid.txt') is Error(Fs.FsError.NotDirectory('invalid.txt', _)));
  const Temporary := Fs.CreateTempFile().Unwrap();
  discard Fs.DeleteFile(Temporary).Unwrap();
  Test.AssertTrue(Proc.Run('__fpas_missing_command_ap475__', []) is Error(Proc.ProcError.StartFailed(Errors.HostError.Failure('Std.Proc.Run', _, Some(_), Some(_), _))));
  Test.AssertTrue(Proc.RunCapture('__fpas_missing_command_ap475__', []) is Error(Proc.ProcError.StartFailed(_)));
  Test.AssertTrue(Proc.Run('bad'#0, []) is Error(Proc.ProcError.InvalidArgument('Command', _)));
  Test.AssertTrue(Parse.Real('1.0e9999') is Error(Parse.ParseError.OutOfRange('real', _)));
  Test.AssertTrue(Parse.Real('1.0e-9999x') is Error(Parse.ParseError.InvalidSyntax('real', _)));
  Test.AssertTrue(Parse.Bool('1') is Error(Parse.ParseError.InvalidSyntax('boolean', _)));
  Test.AssertTrue(Json.Parse('{') is Error(Json.JsonError.InvalidSyntax(Some(1), Some(1), _)));
  Test.AssertTrue(Toml.Parse('x = [') is Error(Toml.TomlError.InvalidSyntax(Some(1), Some(_), _)));
  Test.AssertTrue(JsonFields.ParseObject('{') is Error(JsonFields.FieldError.ParseFailure(Json.JsonError.InvalidSyntax(Some(1), Some(1), _))));
  Test.AssertTrue(TomlFields.ParseTable('x = [') is Error(TomlFields.FieldError.ParseFailure(Toml.TomlError.InvalidSyntax(Some(1), Some(_), _))));
  Test.AssertTrue(JsonFields.ParseObject('[]') is Error(JsonFields.FieldError.ExpectedObject));
  Test.AssertTrue(Crypto.RandomInt(2, 1) is Error(Crypto.CryptoError.InvalidArgument('Lo', _)));
  Test.AssertTrue(Crypto.RandomBytes(-1) is Error(Crypto.CryptoError.InvalidArgument('Count', _)));
end."#,
    );
    fs::write(f.0.join("invalid.txt"), [0xff]).expect("invalid UTF-8 fixture");
    let large = fs::File::create(f.0.join("large.txt")).expect("sparse fixture");
    large.set_len(67_108_865).expect("sparse file size");
    fs::create_dir(f.0.join("directory")).expect("directory");
    fs::write(f.0.join("directory/entry"), "x").expect("directory entry");
    fs::write(f.0.join("app.fpasprj"), "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n").expect("manifest");
    f.run(&["run", "app.fpasprj"], 0);
    f.run(&["build", "app.fpasprj"], 0);
    f.run(&["run", "app.fpascp"], 0);
}

#[test]
fn task_network_and_http_causes_survive_portable_images() {
    let f = Fixture::new(
        r#"program NetworkErrors;
uses Std.Net as Net, Std.Tasks as Tasks, Std.Server as Server, Std.Errors as Errors,
  Std.Http as Http, Std.Net.Utf8 as Utf8, Std.Conv as Conv, Std.Test as Test;
begin
  const Listener := Net.Listen('127.0.0.1', 0).Unwrap();
  const Address := Net.ListenerLocalAddress(Listener).Unwrap();
  case Net.Listen(Address.Host, Address.Port) of
    when Ok(_): Test.Fail('address already in use was accepted');
    when Error(Net.NetError.AddressInUse(Errors.HostError.Failure('Std.Net.Listen', const Resources, Some(_), Some(_), _))):
      Test.AssertEquals(2, Resources.Length());
      Test.AssertEquals(Address.Host, Resources[0]);
      Test.AssertEquals(Conv.IntToStr(Address.Port), Resources[1]);
    when Error(_): Test.Fail('address-in-use cause was lost');
  end case;
  const Client := Net.Connect(Address.Host, Address.Port, 2000).Unwrap();
  const Peer := Net.Accept(Listener).Unwrap();
  discard Net.SetTimeout(Peer, 100).Unwrap();
  Test.AssertTrue(Net.ReceiveBytes(Peer, 1) is Error(Net.NetError.Timeout(Errors.HostError.Failure('Std.Net.ReceiveBytes', _, Some(_), Some(_), _))));
  Test.AssertTrue(Net.SendBytes(Client, [-1]) is Error(Net.NetError.InvalidArgument('Data', _)));
  const Source := Tasks.CreateCancellationSource();
  const Token := Tasks.GetCancellationToken(Source);
  discard Tasks.Cancel(Source);
  Test.AssertTrue(Net.ReceiveBytesWithCancellation(Peer, 1, Token) is Error(Net.NetError.Cancelled('Std.Net.ReceiveBytesWithCancellation')));
  Test.AssertTrue(Net.ConnectWithCancellation(Address.Host, Address.Port, 2000, Token) is Error(Net.NetError.Cancelled('Std.Net.ConnectWithCancellation')));
  discard Net.Close(Client).Unwrap();
  Test.AssertEquals(0, Net.ReceiveBytes(Peer, 1).Unwrap().Length());
  discard Net.Close(Peer).Unwrap();
  Test.AssertTrue(Net.ReceiveBytes(Peer, 1) is Error(Net.NetError.Closed('Std.Net.ReceiveBytes')));
  discard Net.CloseListener(Listener).Unwrap();
  Test.AssertTrue(Net.Connect(Address.Host, Address.Port, 1000) is Error(Net.NetError.ConnectionRefused(Errors.HostError.Failure('Std.Net.Connect', _, Some(_), Some(_), _))));
  Test.AssertTrue(Http.Send(Http.Request.Get('ftp://example.test/')) is Error(Http.HttpError.Address(_)));
  const Decoder := Http.CreateSseDecoder(100).Unwrap();
  discard Http.FeedSse(Decoder, [100, 97, 116, 97, 58, 32, 226]).Unwrap();
  Test.AssertTrue(Http.FinishSse(Decoder) is Error(Http.HttpError.Encoding(Utf8.Utf8Error.IncompleteSequence(6))));
  const Life := Server.CreateLifetime(0, false).Unwrap();
  Test.AssertTrue(Server.OwnListener(Life, Listener) is Error(Server.ServerError.ListenerFailure(Net.NetError.Closed('Std.Server.OwnListener'))));
  Test.AssertTrue(Server.FinishShutdown(Life) is Error(Server.ServerError.InvalidState('Std.Server.FinishShutdown')));
  discard Server.RequestStop(Life);
  Test.AssertTrue(Server.FinishShutdown(Life) is Error(Server.ServerError.Timeout('Std.Server.FinishShutdown')));
  discard Tasks.CloseTaskGroup(Server.GetWorkGroup(Life));
  Test.AssertTrue(Server.FinishShutdown(Life).Unwrap());
end."#,
    );
    fs::write(f.0.join("app.fpasprj"), "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n").expect("manifest");
    f.run(&["run", "app.fpasprj"], 0);
    f.run(&["build", "app.fpasprj"], 0);
    f.run(&["run", "app.fpascp"], 0);
}

#[test]
fn runtime_errors_keep_layouts_without_source_constructor_references() {
    let f = Fixture::new(
        r#"program NativeLayouts;
uses Std.Tasks as Tasks, Std.Net as Net, Std.Server as Server, Std.Test as Test;
begin
  const Queue: channel of integer := Tasks.CreateChannel(1);
  discard Tasks.CloseChannel(Queue);
  Test.AssertTrue(Tasks.Receive(Queue) is Error(_));
  Test.AssertTrue(Net.Connect('localhost', 0, 10) is Error(_));
  const Life := Server.CreateLifetime(1000, false).Unwrap();
  const Listener := Net.Listen('127.0.0.1', 0).Unwrap();
  discard Server.OwnListener(Life, Listener).Unwrap();
  discard Net.CloseListener(Listener).Unwrap();
  discard Server.RequestStop(Life);
  discard Tasks.CloseTaskGroup(Server.GetWorkGroup(Life));
  Test.AssertEquals(1, Server.ShutdownErrors(Life).Length());
  Test.AssertTrue(Server.FinishShutdown(Life).Unwrap());
end."#,
    );
    fs::write(f.0.join("app.fpasprj"), "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n").expect("manifest");
    f.run(&["run", "app.fpasprj"], 0);
    f.run(&["build", "app.fpasprj"], 0);
    f.run(&["run", "app.fpascp"], 0);
}

#[test]
fn internal_buffered_http_parser_reports_truncated_chunked_bodies() {
    let f = Fixture::new(
        r#"program ChunkedEnd;
uses Std.Http.Wire as Wire, Std.Http.Types as Types, Std.Net.Utf8 as Utf8, Std.Test as Test;
begin
  const CrLf := string.Chr(13) + string.Chr(10);
  const Head := 'HTTP/1.1 200 OK' + CrLf + 'Transfer-Encoding: chunked' + CrLf + CrLf;
  Test.AssertTrue(Wire.ParseResponse(Utf8.Encode(Head), 'GET') is
    Error(Types.HttpError.UnexpectedEnd('Std.Http.Wire.DecodeChunked')));
  Test.AssertTrue(Wire.ParseResponse(Utf8.Encode(Head + '1' + CrLf + 'a' + CrLf), 'GET') is
    Error(Types.HttpError.UnexpectedEnd('Std.Http.Wire.DecodeChunked')));
end."#,
    );
    // Compile the real private owners inside this fixture's project, without exporting them.
    let library = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lib/Std");
    fs::create_dir_all(f.0.join("Std/Http")).expect("HTTP owners");
    for entry in fs::read_dir(library.join("Http")).expect("HTTP sources") {
        let entry = entry.expect("source entry");
        if entry.path().extension().is_some_and(|ext| ext == "fpas") {
            fs::copy(entry.path(), f.0.join("Std/Http").join(entry.file_name()))
                .expect("HTTP owner source");
        }
    }
    fs::create_dir_all(f.0.join("Std/Net")).expect("network owners");
    for file in ["Uri.fpas", "Utf8.fpas", "Decimal.fpas"] {
        fs::copy(
            library.join("Net").join(file),
            f.0.join("Std/Net").join(file),
        )
        .expect("network owner source");
    }
    fs::write(f.0.join("app.fpasprj"), "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['main.fpas']\n").expect("manifest");
    fs::write(f.0.join("stdlib.fpasprj"), "[project]\nname = 'http-owner-fixture'\nkind = 'library'\n[exports]\nunits = ['Std.Http.Wire', 'Std.Http.Types', 'Std.Net.Utf8']\n[sources]\ninclude = ['Std/**/*.fpas']\n").expect("owner library manifest");
    f.run(&["run", "--std-lib", ".", "app.fpasprj"], 0);
    f.run(&["build", "--std-lib", ".", "app.fpasprj"], 0);
    f.run(&["run", "--std-lib", ".", "app.fpascp"], 0);
}
