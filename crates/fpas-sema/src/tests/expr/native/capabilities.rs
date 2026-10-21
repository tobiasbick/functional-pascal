//! Native storage and `var` calls preserve conservative transfer evidence.

use super::{check_errors, check_ok};
use crate::TransferCapability;
use fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE;

fn program(declarations: &str, body: &str) -> String {
    format!(
        "program Capabilities; uses Std.Tasks as Tasks;
        type Handler = function(): integer;
        type Bundle = record Items: array of Handler; Work: Handler; Number: integer; end record;
        function Free(): integer; begin return 1; end function;
        {declarations}
        procedure Main(); begin
          var Count := 0;
          const Local: Handler := function(): integer
            begin Count := Count + 1; return Count; end function;
          {body}
        end procedure;
        begin Main(); end."
    )
}

fn rejects_local(body: &str) {
    let errors = check_errors(&program("", body));
    assert!(!errors.is_empty(), "expected local evidence: {body}");
    assert!(
        errors
            .iter()
            .all(|error| error.code == SEMA_TASK_BOUND_CALLABLE),
        "unexpected diagnostics for {body}: {errors:#?}"
    );
}

#[test]
fn stored_native_arguments_taint_whole_and_nested_in_place_receivers() {
    for operation in [
        "Items.PushInPlace(Local);",
        "Items.ConcatInPlace(B := [Local]);",
    ] {
        rejects_local(&format!(
            "var Items: array of Handler := []; {operation}
             const Queue: channel of array of Handler := Tasks.CreateChannel(1);
             discard Tasks.Send(Queue, Items);"
        ));
    }
    rejects_local(
        "var Box := Bundle(Items := [], Work := Free, Number := 0);
         Box.Items.PushInPlace(Value := Local);
         Box.Number := 1;
         const Queue: channel of Bundle := Tasks.CreateChannel(1);
         discard Tasks.Send(Queue, Box);",
    );
    rejects_local(
        "var Values: dict of string to Handler := [:];
         Values.MergeInPlace(D2 := ['local': Local]);
         const Queue: channel of dict of string to Handler := Tasks.CreateChannel(1);
         discard Tasks.Send(Queue, Values);",
    );
}

#[test]
fn native_value_operations_preserve_stored_callable_inputs() {
    for expression in [
        "array.Fill(Count := 2, Value := Local)",
        "[Free].Push(Value := Local)",
        "[Free].Concat([Local])",
        "[Local].Reverse().Slice(0, 1)",
        "['local': Local].Values()",
    ] {
        rejects_local(&format!(
            "const Queue: channel of array of Handler := Tasks.CreateChannel(1);
             discard Tasks.Send(Queue, {expression});"
        ));
    }
    for expression in ["Some(Local).Unwrap()", "[Local].Pop().Value"] {
        rejects_local(&format!(
            "const Queue: channel of Handler := Tasks.CreateChannel(1);
             discard Tasks.Send(Queue, {expression});"
        ));
    }
    rejects_local(
        "const Queue: channel of Option of Handler := Tasks.CreateChannel(1);
         discard Tasks.Send(Queue, ['local': Local].Get('local'));",
    );
}

#[test]
fn invoked_local_callbacks_do_not_taint_retained_transferable_values() {
    check_ok(&program(
        "",
        "var Items: array of Handler := [Free];
         Items.FilterInPlace(function(Item: Handler): boolean
           begin Count := Count + 1; return true; end function);
         Items.SortByInPlace(function(Item: Handler): integer
           begin Count := Count + 1; return Count; end function);
         const Queue: channel of array of Handler := Tasks.CreateChannel(1);
         discard Tasks.Send(Queue, Items);
         discard Tasks.Send(Queue, [Free].Filter(function(Item: Handler): boolean
           begin Count := Count + 1; return true; end function));
         discard Tasks.Send(Queue, [Free].SortBy(function(Item: Handler): integer
           begin Count := Count + 1; return Count; end function));
         discard Tasks.Send(Queue, [Local].Map(function(Item: Handler): Handler
           begin return Free; end function));
         discard Tasks.Send(Queue, [Free].Map(function(Item: Handler): Handler
           begin Count := Count + 1; return Free; end function));",
    ));
}

#[test]
fn known_local_callback_results_taint_mapping_and_reduction_outputs() {
    for expression in [
        "[Free].Map(function(Item: Handler): Handler begin return Local; end function)",
        "[Free].FlatMap(function(Item: Handler): array of Handler begin return [Local]; end function)",
    ] {
        rejects_local(&format!(
            "const Queue: channel of array of Handler := Tasks.CreateChannel(1);
             discard Tasks.Send(Queue, {expression});"
        ));
    }
    rejects_local(
        "var Items: array of Handler := [Free];
         Items.MapInPlace(function(Item: Handler): Handler begin return Local; end function);
         const Queue: channel of array of Handler := Tasks.CreateChannel(1);
         discard Tasks.Send(Queue, Items);",
    );
    rejects_local(
        "const Queue: channel of Handler := Tasks.CreateChannel(1);
         discard Tasks.Send(Queue, [Free].Reduce(F := function(Acc: Handler; Item: Handler): Handler
           begin return Local; end function, Init := Free));",
    );
}

#[test]
fn in_place_writes_leave_prior_value_copies_transferable() {
    check_ok(&program(
        "",
        "var Items: array of Handler := [Free]; const Before := Items;
         Items.PushInPlace(Local);
         const Queue: channel of array of Handler := Tasks.CreateChannel(1);
         discard Tasks.Send(Queue, Before);",
    ));
}

fn snapshot_capability(body: &str) -> TransferCapability {
    let source = program(
        "procedure Touch(var Value: Handler); begin null; end procedure;
         procedure TouchScalar(var Value: integer); begin Value := 1; end procedure;",
        &format!("{body} const Observer := procedure() begin discard Snapshot; end procedure;"),
    );
    let (parsed, errors) = fpas_parser::parse(&source);
    assert!(errors.is_empty(), "{errors:#?}");
    let metadata = crate::analyze_with_types(&parsed);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    metadata
        .closure_infos
        .values()
        .find(|info| {
            info.captures
                .iter()
                .any(|capture| capture.name.eq_ignore_ascii_case("Snapshot"))
        })
        .expect("observer capture")
        .capability
}

#[test]
fn var_calls_invalidate_transferable_callable_targets_without_erasing_local_evidence() {
    assert_eq!(
        snapshot_capability(
            "var Chosen: Handler := Free; Touch(var Chosen); const Snapshot := Chosen;"
        ),
        TransferCapability::Unknown,
    );
    assert_eq!(
        snapshot_capability(
            "var Chosen: Handler := Local; Touch(Value := var Chosen); const Snapshot := Chosen;"
        ),
        TransferCapability::TaskBound,
    );
    assert_eq!(
        snapshot_capability(
            "var Box := Bundle(Items := [], Work := Free, Number := 0); Touch(var Box.Work); const Snapshot := Box;"
        ),
        TransferCapability::Unknown,
    );
    assert_eq!(
        snapshot_capability(
            "var Box := Bundle(Items := [], Work := Free, Number := 0); TouchScalar(var Box.Number); const Snapshot := Box;"
        ),
        TransferCapability::Transferable,
    );
}

#[test]
fn mapping_in_place_retains_unknown_callback_result_capability() {
    assert_eq!(
        snapshot_capability(
            "var Items: array of Handler := [Free];
             Items.MapInPlace(function(Item: Handler): Handler begin return Item; end function);
             const Snapshot := Items;"
        ),
        TransferCapability::Unknown,
    );
}
