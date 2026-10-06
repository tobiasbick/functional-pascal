use super::assert_succeeds;

#[test]
fn task_token_cancels_network_read_end_to_end() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind silent peer");
    let port = listener.local_addr().expect("address").port();
    assert_succeeds(&format!(
        "\
program CancellableRead;
uses Std.Net, Std.Tasks, Std.Time;

function ReadUntilCancelled(ConnectionValue: Std.Net.Connection;
  Token: Std.Tasks.CancellationToken): string;
begin
  case Std.Net.ReceiveBytesWithCancellation(ConnectionValue, 1, Token) of
    Ok(Data): return 'received';
    Error(Message): return Message;
  end;
end function;

begin
  case Std.Net.Connect('127.0.0.1', {port}, 1000) of
    Ok(ConnectionValue):
    begin
      Std.Net.SetTimeout(ConnectionValue, 1000);
      var Source: Std.Tasks.CancellationSource := Std.Tasks.CreateCancellationSource();
      var Token: Std.Tasks.CancellationToken := Std.Tasks.GetCancellationToken(Source);
      var Waiting: task := go ReadUntilCancelled(ConnectionValue, Token);
      Std.Time.Sleep(30);
      Std.Tasks.Cancel(Source);
      if Std.Tasks.Wait(Waiting) <> 'Network read cancelled' then
        panic('read did not report cancellation'); end if;
      case Std.Net.Close(ConnectionValue) of
        Ok(Closed): if not Closed then panic('close failed'); end if;
        Error(Message): panic(Message);
      end;
    end;
    Error(Message): panic(Message);
  end;
end."
    ));
}
