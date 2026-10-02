use super::assert_succeeds;

#[test]
fn task_token_cancels_network_read_end_to_end() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind silent peer");
    let port = listener.local_addr().expect("address").port();
    assert_succeeds(&format!(
        "program CancellableRead;\nuses Std.Net as Net; uses Std.Tasks as Tasks; uses Std.Time as Time;\n\nfunction ReadUntilCancelled(ConnectionValue: Net.Connection;\n  Token: Tasks.CancellationToken): string;\nbegin\n  case Net.ReceiveBytesWithCancellation(ConnectionValue, 1, Token) of\n    when Ok(Data): return 'received';\n    when Error(Message): return Message;\n  end case;\nend function;\n\nbegin\n  case Net.Connect('127.0.0.1', {port}, 1000) of\n    when Ok(ConnectionValue):\n    begin\n      Net.SetTimeout(ConnectionValue, 1000);\n      var Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();\n      var Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);\n      var Waiting: task := go ReadUntilCancelled(ConnectionValue, Token);\n      Time.Sleep(30);\n      Tasks.Cancel(Source);\n      if Tasks.Wait(Waiting) <> 'Network read cancelled' then\n        panic('read did not report cancellation'); end if;\n      case Net.Close(ConnectionValue) of\n        when Ok(Closed): if not Closed then panic('close failed'); end if;\n        when Error(Message): panic(Message);\n      end case;\n    end;\n    when Error(Message): panic(Message);\n  end case;\nend program;"
    ));
}
