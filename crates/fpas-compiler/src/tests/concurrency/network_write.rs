use std::io::Read;
use std::time::Duration;

use super::assert_succeeds;

#[test]
fn network_write_reports_progress_and_pre_cancellation_end_to_end() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("address").port();
    assert_succeeds(&format!(
        "program CancellableWrite;\nuses Std.Net as Net; uses Std.Tasks as Tasks;\nbegin\n  case Net.Connect('127.0.0.1', {port}, 1000) of\n    when Result.Ok(const ConnectionValue):\n    begin\n      var Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();\n      var Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);\n      case Net.SendBytesWithCancellation(ConnectionValue, [42, 43], Token) of\n        when Result.Ok(const Count): if Count <> 2 then panic('wrong write count'); end if;\n        when Result.Error(const Message): panic(Message);\n      end case;\n      discard Tasks.Cancel(Source);\n      case Net.SendBytesWithCancellation(ConnectionValue, [99], Token) of\n        when Result.Ok(const Count): panic('cancelled write succeeded');\n        when Result.Error(const Message): if Message <> 'Network write cancelled' then panic(Message); end if;\n      end case;\n      discard Net.Close(ConnectionValue);\n    end;\n    when Result.Error(const Message): panic(Message);\n  end case;\nend program;"
    ));
    let (mut peer, _) = listener.accept().expect("accept");
    peer.set_read_timeout(Some(Duration::from_secs(2)))
        .expect("timeout");
    let mut bytes = Vec::new();
    peer.read_to_end(&mut bytes).expect("read");
    assert_eq!(bytes, [42, 43]);
}
