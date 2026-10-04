use super::assert_succeeds;

#[test]
fn network_connect_cancellation_variants_execute_end_to_end() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("address").port();
    assert_succeeds(&format!(
        r#"program CancellableConnect;
uses Std.Net as Net; uses Std.Tasks as Tasks;
begin
  const Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();
  const Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);
  case Net.ConnectWithCancellation('127.0.0.1', {port}, 1000, Token) of
    when Result.Ok(const ConnectionValue): discard Net.Close(ConnectionValue);
    when Result.Error(const Message): panic(Message);
  end case;
  discard Tasks.Cancel(Source);
  case Net.ConnectWithCancellation('unused.invalid', 1, 1000, Token) of
    when Result.Ok(const ConnectionValue): panic('cancelled TCP connect succeeded');
    when Result.Error(const Message): if Message <> 'Network connect cancelled' then panic(Message); end if;
  end case;
  case Net.ConnectTlsWithCancellation('unused.invalid', 1, 1000, Token) of
    when Result.Ok(const ConnectionValue): panic('cancelled TLS connect succeeded');
    when Result.Error(const Message): if Message <> 'Network connect cancelled' then panic(Message); end if;
  end case;
end program;"#
    ));
}
