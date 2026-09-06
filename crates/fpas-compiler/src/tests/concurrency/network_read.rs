use super::assert_succeeds;

#[test]
fn task_token_cancels_network_read_end_to_end() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind silent peer");
    let port = listener.local_addr().expect("address").port();
    assert_succeeds(&format!(
        "\
program CancellableRead;
uses Std.Net, Std.Task, Std.Time;

function ReadUntilCancelled(ConnectionValue: Std.Net.Connection;
  Token: Std.Task.CancellationToken): string;
begin
  case Std.Net.ReadWithCancellation(ConnectionValue, 1, Token) of
    Ok(Data): return 'received';
    Error(Message): return Message
  end
end;

begin
  case Std.Net.Connect('127.0.0.1', {port}, 1000) of
    Ok(ConnectionValue):
    begin
      Std.Net.SetTimeout(ConnectionValue, 1000);
      var Source: Std.Task.CancellationSource := Std.Task.CreateCancellationSource();
      var Token: Std.Task.CancellationToken := Std.Task.GetCancellationToken(Source);
      var Waiting: task := go ReadUntilCancelled(ConnectionValue, Token);
      Std.Time.Sleep(30);
      Std.Task.Cancel(Source);
      if Std.Task.Wait(Waiting) <> 'Network read cancelled' then
        panic('read did not report cancellation');
      case Std.Net.Close(ConnectionValue) of
        Ok(Closed): if not Closed then panic('close failed');
        Error(Message): panic(Message)
      end
    end;
    Error(Message): panic(Message)
  end
end."
    ));
}
