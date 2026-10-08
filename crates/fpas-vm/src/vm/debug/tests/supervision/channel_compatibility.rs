//! Select and ordinary channel calls must cooperate even with a single pool worker.

const PRODUCER: &str = r#"program ChannelCompatibility;
uses Std.Tasks;
begin
  const Group: TaskGroup := CreateTaskGroup();
  const Queue: channel of integer := CreateChannel(1);
  const Child: task := StartTaskInGroup(Group, procedure(Token: CancellationToken)
  begin
    for Item: integer := 1 to 2 do
      discard (SEND_OPERATION).Unwrap(); end for;
  end procedure);
  var Total: integer := 0;
  for Item: integer := 1 to 2 do
    discard Select([
      ReceiveCase(Queue, procedure(Outcome: result of integer, string)
        begin Total := Total + Outcome.Unwrap(); end procedure),
      TimerCase(1000, procedure() begin panic('consumer stalled'); end procedure)
    ]); end for;
  Wait(Child);
  if Total <> 3 then panic('delivery lost'); end if;
  if CloseTaskGroup(Group).Length() <> 0 then panic('worker failed'); end if;
  discard CloseChannel(Queue);
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
uses Std.Tasks;
begin
  const Group: TaskGroup := CreateTaskGroup();
  const Queue: channel of integer := CreateChannel(1);
  const Child: task := StartTaskInGroup(Group, function(Token: CancellationToken): integer
  begin
    var Total: integer := 0;
    for Item: integer := 1 to 2 do Total := Total + (RECEIVE_OPERATION).Unwrap(); end for;
    return Total;
  end function);
  for Item: integer := 1 to 2 do
    discard Select([SendCase(Queue, Item, procedure(Outcome: result of boolean, string)
      begin discard Outcome.Unwrap(); end procedure)]); end for;
  if Wait(Child) <> 3 then panic('delivery lost'); end if;
  if CloseTaskGroup(Group).Length() <> 0 then panic('worker failed'); end if;
  discard CloseChannel(Queue);
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
        "for Item: integer := 1 to 2 do\n      discard (SEND_OPERATION).Unwrap(); end for;",
        "const Grandchild: task := StartTaskInGroup(Group, procedure(ChildToken: CancellationToken)
         begin discard Send(Queue, 1); discard Send(Queue, 2); end procedure);
         WAIT_OPERATION;",
    );
    for operation in [
        "Wait(Grandchild)",
        "WaitAll([Grandchild])",
        "discard WaitAny([Grandchild])",
        "discard WaitAnyWithTimeout([Grandchild], 1000).Unwrap()",
        "discard WaitAnyWithCancellation([Grandchild], Token).Unwrap()",
    ] {
        super::run_with_children(&source.replace("WAIT_OPERATION", operation), 2);
    }
}

#[test]
fn closing_a_nested_group_releases_the_consumers_stack() {
    let source = PRODUCER.replace(
        "for Item: integer := 1 to 2 do\n      discard (SEND_OPERATION).Unwrap(); end for;",
        "const Nested: TaskGroup := CreateTaskGroup();
         const Grandchild: task := StartTaskInGroup(Nested, procedure(ChildToken: CancellationToken)
           begin discard Send(Queue, 1); discard Send(Queue, 2); end procedure);
         if CloseTaskGroup(Nested).Length() <> 0 then panic('nested child failed'); end if;",
    );
    super::run_with_children(&source, 2);
}
