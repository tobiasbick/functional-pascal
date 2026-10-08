//! Deterministic debugger scheduling for deadline-bounded channel operations.

use super::*;

fn timeout_session() -> DebugSession {
    const SOURCE: &str = r#"program DebugChannelTimeout;

uses Std.Tasks;

begin
  const Messages: channel of integer := CreateChannel(1);
  case ReceiveWithTimeout(Messages, 25) of
    when Ok(_): panic('empty channel did not time out');
    when Error(const Message):
      if Message <> 'Channel receive timed out' then panic(Message); end if;
  end case;
  case Send(Messages, 1) of
    when Ok(_): begin end;
    when Error(const Message): panic(Message);
  end case;
  case SendWithTimeout(Messages, 2, 25) of
    when Ok(_): panic('full channel did not time out');
    when Error(const Message):
      if Message <> 'Channel send timed out' then panic(Message); end if;
  end case;
end.
"#;
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    DebugSession::with_manual_clock(
        fpas_compiler::compile(&program).expect("compile channel-timeout fixture"),
    )
    .expect("channel-timeout debug session")
}

#[test]
fn manual_clock_completes_channel_receive_and_send_timeouts() {
    let result = timeout_session()
        .continue_execution()
        .expect("run channel-timeout fixture");
    assert!(matches!(result, DebugRunResult::Terminated(_)));
}
