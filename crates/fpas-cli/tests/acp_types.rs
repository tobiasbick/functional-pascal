//! Detached ACP values survive source interfaces, reused units and portable images.
#![allow(
    clippy::expect_used,
    reason = "integration fixtures assert compile and execution outcomes"
)]

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "fpas-acp-types-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("fixture directory");
        fs::write(root.join("bridge.fpas"), BRIDGE).expect("bridge source");
        fs::write(root.join("main.fpas"), PROGRAM).expect("program source");
        fs::write(root.join("app.fpasprj"), "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n").expect("manifest");
        Self(root)
    }

    fn invoke(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .expect("CLI process")
    }

    fn run(&self, args: &[&str]) {
        let output = self.invoke(args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.stdout.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("fixture cleanup");
    }
}

const BRIDGE: &str = r#"unit Bridge;
uses Std.Ai.Acp.Types as Acp, Std.Ai.Acp.V2 as V2;
public type Failure = Acp.AcpError;
public type Patch = Acp.Field of array of string;
public function Peer(): Failure;
begin
  return Acp.AcpError.Peer(Acp.ProtocolError(Code := -32001, Message := 'preserved', Data := Some(Acp.ParseRawJson('{"n":18446744073709551616}').Unwrap())));
end function;
public function Empty(): Patch;
begin return Acp.Field.Value([]); end function;
public function Request(): Acp.ClientRequest;
begin
  return Acp.ClientRequest.PromptRequest(Acp.PromptRequest(SessionId := Acp.SessionId('cached'), Prompt := [], Meta := None));
end function;
public function Update(): V2.AgentNotification;
begin
  return V2.DecodeAgentNotification('session/update', Acp.ParseRawJson('{"sessionId":"cached","update":{"sessionUpdate":"tool_call_update","toolCallId":"t","title":null,"content":[]}}').Unwrap()).Unwrap().Payload;
end function;
end unit;"#;

const PROGRAM: &str = r#"program CachedAcp;
uses Bridge as B, Std.Ai.Acp.Types as Acp, Std.Ai.Acp.V2 as V2;
begin
  if Acp.EncodeClientRequest('session/prompt', B.Request()).Unwrap().Text <> '{"sessionId":"cached","prompt":[]}' then panic('request identity'); end if;
  const Updated := V2.EncodeAgentNotification('session/update', B.Update()).Unwrap().Text;
  if not ('"title":null' in Updated) or not ('"content":[]' in Updated) then panic('v2 patches lost'); end if;
  const Failure: Acp.AcpError := B.Peer();
  if ('preserved' in Failure.Message()) or ('18446744073709551616' in Failure.Message()) then panic('peer payload leaked'); end if;
  const Patch: Acp.Field of array of string := B.Empty();
  case Patch of
    when Acp.Field.Value(const Items):
      if Items.Length() <> 0 then panic('empty replacement'); end if;
    when Acp.Field.Missing, Acp.Field.NullValue: panic('presence lost');
  end case;
  case Failure of
    when Acp.AcpError.Peer(const Protocol):
      if (Protocol.Code <> -32001) or (Protocol.Message <> 'preserved') then panic('structured peer identity'); end if;
      if Acp.RawJsonText(Protocol.Data.Unwrap()).Unwrap() <> '{"n":18446744073709551616}' then panic('raw data lost'); end if;
    when Acp.AcpError.InvalidArgument(_, _), Acp.AcpError.UnsupportedCapability(_), Acp.AcpError.UnsupportedVersion(_), Acp.AcpError.Busy(_), Acp.AcpError.Closed(_), Acp.AcpError.Timeout(_), Acp.AcpError.Cancelled(_), Acp.AcpError.LimitExceeded(_, _), Acp.AcpError.InvalidMessage(_, _, _), Acp.AcpError.Host(_): panic('peer lost');
  end case;
end."#;

#[test]
fn peer_record_payloads_and_generic_patches_survive_cached_units_and_source_free_images() {
    let fixture = Fixture::new();
    fixture.run(&["run", "app.fpasprj"]);
    let cache = fixture.0.join("bridge.fpascu");
    let bytes = fs::read(&cache).expect("compiled bridge");
    let modified = fs::metadata(&cache)
        .expect("cache metadata")
        .modified()
        .expect("cache time");
    fs::write(
        fixture.0.join("main.fpas"),
        format!("{PROGRAM}\n// Rebuild the caller using the unchanged bridge.\n"),
    )
    .expect("changed caller");
    fixture.run(&["run", "app.fpasprj"]);
    assert_eq!(bytes, fs::read(&cache).expect("reused sidecar"));
    assert_eq!(
        modified,
        fs::metadata(&cache)
            .expect("cache metadata")
            .modified()
            .expect("cache time")
    );
    for name in ["main.fpas", "bridge.fpas", "bridge.fpascu"] {
        fs::remove_file(fixture.0.join(name)).expect("remove fixture source or cache");
    }
    fixture.run(&["run", "--std-lib", "absent-library", "app.fpascp"]);
}

#[test]
fn structured_acp_payloads_keep_nominal_type_checking() {
    let fixture = Fixture::new();
    fixture.run(&["run", "app.fpasprj"]);
    fs::write(fixture.0.join("main.fpas"), "program Wrong; uses Bridge as B, Std.Ai.Acp.Types as Acp; begin const Patch: Acp.Field of integer := B.Empty(); end.").expect("wrong family caller");
    let output = fixture.invoke(&["check", "app.fpasprj"]);
    assert!(
        !output.status.success(),
        "an array patch cannot become an integer patch"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("FP3006"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn protocol_families_and_distinct_ids_are_not_interchangeable() {
    let fixture = Fixture::new();
    for body in [
        "discard Acp.EncodeClientResponse('session/prompt', B.Request());",
        "const Wrong: V2.ClientRequest := B.Request();",
        "const Wrong: Acp.ToolCallId := Acp.SessionId('s');",
    ] {
        fs::write(fixture.0.join("main.fpas"), format!("program Wrong; uses Bridge as B, Std.Ai.Acp.Types as Acp, Std.Ai.Acp.V2 as V2; begin {body} end.")).expect("write invalid ACP type fixture");
        let output = fixture.invoke(&["check", "app.fpasprj"]);
        assert!(!output.status.success(), "{body}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("FP3006"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
