use std::io::Read;
use std::time::Duration;

use super::assert_succeeds;

#[test]
fn network_write_reports_progress_and_pre_cancellation_end_to_end() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("address").port();
    assert_succeeds(&format!(
        r#"program CancellableWrite;
uses Std.Net as Net; uses Std.Tasks as Tasks;
begin
  case Net.Connect('127.0.0.1', {port}, 1000) of
    when Result.Ok(const ConnectionValue):
    begin
      const Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();
      const Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);
      case Net.SendBytesWithCancellation(ConnectionValue, [42, 43], Token) of
        when Result.Ok(const Count): if Count <> 2 then panic('wrong write count'); end if;
        when Result.Error(const Message): panic(Message);
      end case;
      discard Tasks.Cancel(Source);
      case Net.SendBytesWithCancellation(ConnectionValue, [99], Token) of
        when Result.Ok(const Count): panic('cancelled write succeeded');
        when Result.Error(const Message): if Message <> 'Network write cancelled' then panic(Message); end if;
      end case;
      discard Net.Close(ConnectionValue);
    end;
    when Result.Error(const Message): panic(Message);
  end case;
end program;"#
    ));
    let (mut peer, _) = listener.accept().expect("accept");
    peer.set_read_timeout(Some(Duration::from_secs(2)))
        .expect("timeout");
    let mut bytes = Vec::new();
    peer.read_to_end(&mut bytes).expect("read");
    assert_eq!(bytes, [42, 43]);
}
