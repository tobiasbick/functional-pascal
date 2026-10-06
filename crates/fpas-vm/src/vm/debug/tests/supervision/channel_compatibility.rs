//! Select and ordinary channel calls must cooperate even with a single pool worker.

const PRODUCER: &str = r#"program ChannelCompatibility;
uses Std.Tasks, Std.Results, Std.Arrays;
begin
  var Group: TaskGroup := CreateTaskGroup();
  var Queue: channel of integer := CreateChannel(1);
  var Child: task := StartTaskInGroup(Group, procedure(Token: CancellationToken)
  begin
    for Item: integer := 1 to 2 do
      Unwrap(SEND_OPERATION); end for;
  end procedure);
  mutable var Total: integer := 0;
  for Item: integer := 1 to 2 do
    Select([
      ReceiveCase(Queue, procedure(Outcome: result of integer, string)
        begin Total := Total + Unwrap(Outcome); end procedure),
      TimerCase(1000, procedure() begin panic('consumer stalled'); end procedure)
    ]); end for;
  Wait(Child);
  if Total <> 3 then panic('delivery lost'); end if;
  if Length(CloseTaskGroup(Group)) <> 0 then panic('worker failed'); end if;
  CloseChannel(Queue);
end."#;

#[test]
fn ordinary_send_cooperates_with_a_select_consumer() {
    super::run_both(&PRODUCER.replace("SEND_OPERATION", "Send(Queue, Item)"));
}

#[test]
fn cancellable_send_cooperates_with_a_select_consumer() {
    super::run_both(
        &PRODUCER.replace("SEND_OPERATION", "SendWithCancellation(Queue, Item, Token)"),
    );
}

#[test]
fn timed_send_cooperates_with_a_select_consumer() {
    super::run_both(&PRODUCER.replace("SEND_OPERATION", "SendWithTimeout(Queue, Item, 1000)"));
}

const CONSUMER: &str = r#"program ReceiveCompatibility;
uses Std.Tasks, Std.Results, Std.Arrays;
begin
  var Group: TaskGroup := CreateTaskGroup();
  var Queue: channel of integer := CreateChannel(1);
  var Child: task := StartTaskInGroup(Group, function(Token: CancellationToken): integer
  begin
    mutable var Total: integer := 0;
    for Item: integer := 1 to 2 do Total := Total + Unwrap(RECEIVE_OPERATION); end for;
    return Total;
  end function);
  for Item: integer := 1 to 2 do
    Select([SendCase(Queue, Item, procedure(Outcome: result of boolean, string)
      begin Unwrap(Outcome); end procedure)]); end for;
  if Wait(Child) <> 3 then panic('delivery lost'); end if;
  if Length(CloseTaskGroup(Group)) <> 0 then panic('worker failed'); end if;
  CloseChannel(Queue);
end."#;

#[test]
fn ordinary_receive_cooperates_with_a_select_producer() {
    super::run_both(&CONSUMER.replace("RECEIVE_OPERATION", "Receive(Queue)"));
}

#[test]
fn cancellable_receive_cooperates_with_a_select_producer() {
    super::run_both(
        &CONSUMER.replace("RECEIVE_OPERATION", "ReceiveWithCancellation(Queue, Token)"),
    );
}

#[test]
fn timed_receive_cooperates_with_a_select_producer() {
    super::run_both(&CONSUMER.replace("RECEIVE_OPERATION", "ReceiveWithTimeout(Queue, 1000)"));
}

#[test]
fn nested_task_barriers_release_the_consumers_stack() {
    let source = PRODUCER.replace(
        "for Item: integer := 1 to 2 do\n      Unwrap(SEND_OPERATION); end for;",
        "var Grandchild: task := StartTaskInGroup(Group, procedure(ChildToken: CancellationToken)
         begin Send(Queue, 1); Send(Queue, 2); end procedure);
         WAIT_OPERATION;",
    );
    for operation in [
        "Wait(Grandchild)",
        "WaitAll([Grandchild])",
        "WaitAny([Grandchild])",
        "Unwrap(WaitAnyWithTimeout([Grandchild], 1000))",
        "Unwrap(WaitAnyWithCancellation([Grandchild], Token))",
    ] {
        super::run_with_children(&source.replace("WAIT_OPERATION", operation), 2);
    }
}

#[test]
fn closing_a_nested_group_releases_the_consumers_stack() {
    let source = PRODUCER.replace(
        "for Item: integer := 1 to 2 do\n      Unwrap(SEND_OPERATION); end for;",
        "var Nested: TaskGroup := CreateTaskGroup();
         StartTaskInGroup(Nested, procedure(ChildToken: CancellationToken)
           begin Send(Queue, 1); Send(Queue, 2); end procedure);
         if Length(CloseTaskGroup(Nested)) <> 0 then panic('nested child failed'); end if;",
    );
    super::run_with_children(&source, 2);
}
