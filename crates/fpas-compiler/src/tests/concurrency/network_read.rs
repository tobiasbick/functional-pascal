use super::assert_succeeds;

#[test]
fn task_token_cancels_network_read_end_to_end() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind silent peer");
    let port = listener.local_addr().expect("address").port();
    assert_succeeds(&format!(
        r#"program CancellableRead;
uses Std.Net as Net; uses Std.Tasks as Tasks; uses Std.Time as Time;

function ReadUntilCancelled(ConnectionValue: Net.Connection;
  Token: Tasks.CancellationToken): string;
begin
  case Net.ReceiveBytesWithCancellation(ConnectionValue, 1, Token) of
    when Result.Ok(const Data): return 'received';
    when Result.Error(const Message): return Message;
  end case;
end function;

begin
  case Net.Connect('127.0.0.1', {port}, 1000) of
    when Result.Ok(const ConnectionValue):
    begin
      discard Net.SetTimeout(ConnectionValue, 1000);
      const Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();
      const Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);
      const Waiting: task := go ReadUntilCancelled(ConnectionValue, Token);
      Time.Sleep(30);
      discard Tasks.Cancel(Source);
      if Tasks.Wait(Waiting) <> 'Network read cancelled' then
        panic('read did not report cancellation'); end if;
      case Net.Close(ConnectionValue) of
        when Result.Ok(const Closed): if not Closed then panic('close failed'); end if;
        when Result.Error(const Message): panic(Message);
      end case;
    end;
    when Result.Error(const Message): panic(Message);
  end case;
end program;"#
    ));
}
