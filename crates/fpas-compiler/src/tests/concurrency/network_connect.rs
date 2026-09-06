use super::assert_succeeds;

#[test]
fn network_connect_cancellation_variants_execute_end_to_end() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("address").port();
    assert_succeeds(&format!(
        "\
program CancellableConnect;
uses Std.Net, Std.Task;
begin
  var Source: Std.Task.CancellationSource := Std.Task.CreateCancellationSource();
  var Token: Std.Task.CancellationToken := Std.Task.GetCancellationToken(Source);
  case Std.Net.ConnectWithCancellation('127.0.0.1', {port}, 1000, Token) of
    Ok(ConnectionValue): Std.Net.Close(ConnectionValue);
    Error(Message): panic(Message)
  end;
  Std.Task.Cancel(Source);
  case Std.Net.ConnectWithCancellation('unused.invalid', 1, 1000, Token) of
    Ok(ConnectionValue): panic('cancelled TCP connect succeeded');
    Error(Message): if Message <> 'Network connect cancelled' then panic(Message)
  end;
  case Std.Net.ConnectTlsWithCancellation('unused.invalid', 1, 1000, Token) of
    Ok(ConnectionValue): panic('cancelled TLS connect succeeded');
    Error(Message): if Message <> 'Network connect cancelled' then panic(Message)
  end
end."
    ));
}
