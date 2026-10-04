use super::assert_succeeds;

#[test]
fn network_connect_cancellation_variants_execute_end_to_end() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("address").port();
    assert_succeeds(&format!(
        "program CancellableConnect;\nuses Std.Net as Net; uses Std.Tasks as Tasks;\nbegin\n  var Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();\n  var Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);\n  case Net.ConnectWithCancellation('127.0.0.1', {port}, 1000, Token) of\n    when Result.Ok(const ConnectionValue): discard Net.Close(ConnectionValue);\n    when Result.Error(const Message): panic(Message);\n  end case;\n  discard Tasks.Cancel(Source);\n  case Net.ConnectWithCancellation('unused.invalid', 1, 1000, Token) of\n    when Result.Ok(const ConnectionValue): panic('cancelled TCP connect succeeded');\n    when Result.Error(const Message): if Message <> 'Network connect cancelled' then panic(Message); end if;\n  end case;\n  case Net.ConnectTlsWithCancellation('unused.invalid', 1, 1000, Token) of\n    when Result.Ok(const ConnectionValue): panic('cancelled TLS connect succeeded');\n    when Result.Error(const Message): if Message <> 'Network connect cancelled' then panic(Message); end if;\n  end case;\nend program;"
    ));
}
