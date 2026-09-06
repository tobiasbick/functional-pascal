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
uses Std.Net, Std.Task;
begin
  case Std.Net.Connect('127.0.0.1', {port}, 1000) of
    Ok(ConnectionValue):
    begin
      var Source: Std.Task.CancellationSource := Std.Task.CreateCancellationSource();
      var Token: Std.Task.CancellationToken := Std.Task.GetCancellationToken(Source);
      case Std.Net.WriteWithCancellation(ConnectionValue, [42, 43], Token) of
        Ok(Count): if Count <> 2 then panic('wrong write count');
        Error(Message): panic(Message)
      end;
      Std.Task.Cancel(Source);
      case Std.Net.WriteWithCancellation(ConnectionValue, [99], Token) of
        Ok(Count): panic('cancelled write succeeded');
        Error(Message): if Message <> 'Network write cancelled' then panic(Message)
      end;
      Std.Net.Close(ConnectionValue)
    end;
    Error(Message): panic(Message)
  end
end."
    ));
    let (mut peer, _) = listener.accept().expect("accept");
    peer.set_read_timeout(Some(Duration::from_secs(2)))
        .expect("timeout");
    let mut bytes = Vec::new();
    peer.read_to_end(&mut bytes).expect("read");
    assert_eq!(bytes, [42, 43]);
}
