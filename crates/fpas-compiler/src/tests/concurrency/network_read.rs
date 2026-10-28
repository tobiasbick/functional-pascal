use super::assert_succeeds;

#[test]
fn task_token_cancels_network_read_end_to_end() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind silent peer");
    let port = listener.local_addr().expect("address").port();
    assert_succeeds(&format!(
        "program CancellableRead;\nuses Std.Net.Sockets as Net, Std.Tasks as Tasks, Std.Time as Time;\n\nfunction ReadUntilCancelled(ConnectionValue: Net.Connection;\n  Token: Tasks.CancellationToken): result of (array of integer, Net.NetError);\nbegin\n  return Net.ReceiveBytesWithCancellation(ConnectionValue, 1, Token);\nend function;\n\nbegin scope\n  case Net.Connect('127.0.0.1', {port}, 1000) of\n    when Ok(const ConnectionValue):\n    begin\n      discard Net.SetTimeout(ConnectionValue, 1000);\n      const Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();\n      const Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);\n      const Waiting: task of result of (array of integer, Net.NetError) := go ReadUntilCancelled(ConnectionValue, Token);\n      Time.Sleep(30);\n      discard Tasks.Cancel(Source);\n      if not (Tasks.Wait(Waiting) is Error(Net.NetError.Cancelled('Std.Net.Sockets.ReceiveBytesWithCancellation'))) then\n        panic('read did not report cancellation'); end if;\n      case Net.Close(ConnectionValue) of\n        when Ok(const Closed): if not Closed then panic('close failed'); end if;\n        when Error(const Message): panic(Message.Message());\n      end case;\n    end;\n    when Error(const Message): panic(Message.Message());\n  end case;\n end scope; end."
    ));
}
