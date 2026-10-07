use std::io::Read;
use std::time::Duration;

use super::assert_succeeds;

#[test]
fn network_write_reports_progress_and_pre_cancellation_end_to_end() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("address").port();
    assert_succeeds(&format!(
        "\
program CancellableWrite;
uses Std.Net, Std.Tasks;
begin
  case Std.Net.Connect('127.0.0.1', {port}, 1000) of
    when Ok(ConnectionValue):
    begin
      const Source: Std.Tasks.CancellationSource := Std.Tasks.CreateCancellationSource();
      const Token: Std.Tasks.CancellationToken := Std.Tasks.GetCancellationToken(Source);
      case Std.Net.SendBytesWithCancellation(ConnectionValue, [42, 43], Token) of
        when Ok(Count): if Count <> 2 then panic('wrong write count'); end if;
        when Error(Message): panic(Message);
      end case;
      discard Std.Tasks.Cancel(Source);
      case Std.Net.SendBytesWithCancellation(ConnectionValue, [99], Token) of
        when Ok(Count): panic('cancelled write succeeded');
        when Error(Message): if Message <> 'Network write cancelled' then panic(Message); end if;
      end case;
      discard Std.Net.Close(ConnectionValue);
    end;
    when Error(Message): panic(Message);
  end case;
end."
    ));
    let (mut peer, _) = listener.accept().expect("accept");
    peer.set_read_timeout(Some(Duration::from_secs(2)))
        .expect("timeout");
    let mut bytes = Vec::new();
    peer.read_to_end(&mut bytes).expect("read");
    assert_eq!(bytes, [42, 43]);
}
