//! Select and ordinary channel calls must cooperate even with a single pool worker.

const PRODUCER: &str = r#"program ChannelCompatibility;
uses Std.Tasks as Tasks; uses Std.Results as Results; uses Std.Arrays as Arrays;
begin
  var Group: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Queue: channel of (integer) := Tasks.CreateChannel(1);
  var Child: task := Tasks.StartTaskInGroup(Group, procedure(Token: Tasks.CancellationToken)
  begin
    for Item: integer := 1 to 2 do
      discard Results.Unwrap(SEND_OPERATION); end for;
  end procedure);
  mutable var Total: integer := 0;
  for Item: integer := 1 to 2 do
    discard Tasks.Select([
      Tasks.ReceiveCase(Queue, procedure(Outcome: result of (integer, string))
        begin Total := Total + Results.Unwrap(Outcome); end procedure),
      Tasks.TimerCase(1000, procedure() begin panic('consumer stalled'); end procedure)
    ]); end for;
  Tasks.Wait(Child);
  if Total <> 3 then panic('delivery lost'); end if;
  if Arrays.Length(Tasks.CloseTaskGroup(Group)) <> 0 then panic('worker failed'); end if;
  discard Tasks.CloseChannel(Queue);
end program;"#;

#[test]
fn ordinary_send_cooperates_with_a_select_consumer() {
    super::run_both(&PRODUCER.replace("SEND_OPERATION", "Tasks.Send(Queue, Item)"));
}

#[test]
fn cancellable_send_cooperates_with_a_select_consumer() {
    super::run_both(&PRODUCER.replace(
        "SEND_OPERATION",
        "Tasks.SendWithCancellation(Queue, Item, Token)",
    ));
}

#[test]
fn timed_send_cooperates_with_a_select_consumer() {
    super::run_both(
        &PRODUCER.replace("SEND_OPERATION", "Tasks.SendWithTimeout(Queue, Item, 1000)"),
    );
}

const CONSUMER: &str = r#"program ReceiveCompatibility;
uses Std.Tasks as Tasks; uses Std.Results as Results; uses Std.Arrays as Arrays;
begin
  var Group: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Queue: channel of (integer) := Tasks.CreateChannel(1);
  var Child: task := Tasks.StartTaskInGroup(Group, function(Token: Tasks.CancellationToken): integer
  begin
    mutable var Total: integer := 0;
    for Item: integer := 1 to 2 do Total := Total + Results.Unwrap(RECEIVE_OPERATION); end for;
    return Total;
  end function);
  for Item: integer := 1 to 2 do
    discard Tasks.Select([Tasks.SendCase(Queue, Item, procedure(Outcome: result of (boolean, string))
      begin discard Results.Unwrap(Outcome); end procedure)]); end for;
  if Tasks.Wait(Child) <> 3 then panic('delivery lost'); end if;
  if Arrays.Length(Tasks.CloseTaskGroup(Group)) <> 0 then panic('worker failed'); end if;
  discard Tasks.CloseChannel(Queue);
end program;"#;

#[test]
fn ordinary_receive_cooperates_with_a_select_producer() {
    super::run_both(&CONSUMER.replace("RECEIVE_OPERATION", "Tasks.Receive(Queue)"));
}

#[test]
fn cancellable_receive_cooperates_with_a_select_producer() {
    super::run_both(&CONSUMER.replace(
        "RECEIVE_OPERATION",
        "Tasks.ReceiveWithCancellation(Queue, Token)",
    ));
}

#[test]
fn timed_receive_cooperates_with_a_select_producer() {
    super::run_both(
        &CONSUMER.replace("RECEIVE_OPERATION", "Tasks.ReceiveWithTimeout(Queue, 1000)"),
    );
}

#[test]
fn nested_task_barriers_release_the_consumers_stack() {
    let source = PRODUCER.replace(
        "for Item: integer := 1 to 2 do\n      discard Results.Unwrap(SEND_OPERATION); end for;",
        "var Grandchild: task := Tasks.StartTaskInGroup(Group, procedure(ChildToken: Tasks.CancellationToken)
         begin discard Tasks.Send(Queue, 1); discard Tasks.Send(Queue, 2); end procedure);
         WAIT_OPERATION;",
    );
    for operation in [
        "Tasks.Wait(Grandchild)",
        "Tasks.WaitAll([Grandchild])",
        "discard Tasks.WaitAny([Grandchild])",
        "discard Results.Unwrap(Tasks.WaitAnyWithTimeout([Grandchild], 1000))",
        "discard Results.Unwrap(Tasks.WaitAnyWithCancellation([Grandchild], Token))",
    ] {
        super::run_with_children(&source.replace("WAIT_OPERATION", operation), 2);
    }
}

#[test]
fn closing_a_nested_group_releases_the_consumers_stack() {
    let source = PRODUCER.replace(
        "for Item: integer := 1 to 2 do\n      discard Results.Unwrap(SEND_OPERATION); end for;",
        "var Nested: Tasks.TaskGroup := Tasks.CreateTaskGroup();
         var NestedChild: task := Tasks.StartTaskInGroup(Nested, procedure(ChildToken: Tasks.CancellationToken)
           begin discard Tasks.Send(Queue, 1); discard Tasks.Send(Queue, 2); end procedure);
         if Arrays.Length(Tasks.CloseTaskGroup(Nested)) <> 0 then panic('nested child failed'); end if;",
    );
    super::run_with_children(&source, 2);
}
