//! Select and ordinary channel calls must cooperate even with a single pool worker.

const PRODUCER: &str = r#"program ChannelCompatibility;
uses Std.Tasks as Tasks;
begin
  const Group: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  const Queue: channel of integer := Tasks.CreateChannel(1);
  const Child: task := Tasks.StartTaskInGroup(Group, procedure(Token: Tasks.CancellationToken)
  begin
    for Item: integer := 1 to 2 do
      discard (SEND_OPERATION).Unwrap(); end for;
  end procedure);
  var Total: integer := 0;
  for Item: integer := 1 to 2 do
    discard Tasks.Select([
      Tasks.ReceiveCase(Queue, procedure(Outcome: result of (integer, string))
        begin Total := Total + Outcome.Unwrap(); end procedure),
      Tasks.TimerCase(1000, procedure() begin panic('consumer stalled'); end procedure)
    ]); end for;
  Tasks.Wait(Child);
  if Total <> 3 then panic('delivery lost'); end if;
  if Tasks.CloseTaskGroup(Group).Length() <> 0 then panic('worker failed'); end if;
  discard Tasks.CloseChannel(Queue);
end."#;

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
uses Std.Tasks as Tasks;
begin
  const Group: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  const Queue: channel of integer := Tasks.CreateChannel(1);
  const Child: task of integer := Tasks.StartTaskInGroup(Group, function(Token: Tasks.CancellationToken): integer
  begin
    var Total: integer := 0;
    for Item: integer := 1 to 2 do Total := Total + (RECEIVE_OPERATION).Unwrap(); end for;
    return Total;
  end function);
  for Item: integer := 1 to 2 do
    discard Tasks.Select([Tasks.SendCase(Queue, Item, procedure(Outcome: result of (boolean, string))
      begin discard Outcome.Unwrap(); end procedure)]); end for;
  if Tasks.Wait(Child) <> 3 then panic('delivery lost'); end if;
  if Tasks.CloseTaskGroup(Group).Length() <> 0 then panic('worker failed'); end if;
  discard Tasks.CloseChannel(Queue);
end."#;

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
        "for Item: integer := 1 to 2 do\n      discard (SEND_OPERATION).Unwrap(); end for;",
        "const Grandchild: task := Tasks.StartTaskInGroup(Group, procedure(ChildToken: Tasks.CancellationToken)\n         begin discard Tasks.Send(Queue, 1); discard Tasks.Send(Queue, 2); end procedure);\n         WAIT_OPERATION;",
    );
    for operation in [
        "Tasks.Wait(Grandchild)",
        "Tasks.WaitAll([Grandchild])",
        "discard Tasks.WaitAny([Grandchild])",
        "discard Tasks.WaitAnyWithTimeout([Grandchild], 1000).Unwrap()",
        "discard Tasks.WaitAnyWithCancellation([Grandchild], Token).Unwrap()",
    ] {
        super::run_with_children(&source.replace("WAIT_OPERATION", operation), 2);
    }
}

#[test]
fn closing_a_nested_group_releases_the_consumers_stack() {
    let source = PRODUCER.replace(
        "for Item: integer := 1 to 2 do\n      discard (SEND_OPERATION).Unwrap(); end for;",
        "const Nested: Tasks.TaskGroup := Tasks.CreateTaskGroup();\n         const Grandchild: task := Tasks.StartTaskInGroup(Nested, procedure(ChildToken: Tasks.CancellationToken)\n           begin discard Tasks.Send(Queue, 1); discard Tasks.Send(Queue, 2); end procedure);\n         if Tasks.CloseTaskGroup(Nested).Length() <> 0 then panic('nested child failed'); end if;",
    );
    super::run_with_children(&source, 2);
}
