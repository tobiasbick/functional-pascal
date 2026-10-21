//! Source-level static evidence and mandatory guards for unknown capabilities.

use super::*;

#[test]
fn local_factory_results_remain_usable_by_the_creating_task() {
    assert_succeeds(
        "program LocalFactory;
        function Make(): function(): integer;
        begin
          var Value := 40;
          return function(): integer begin Value := Value + 1; return Value; end function;
        end function;
        begin
          const Work := Make();
          if Work() <> 41 then panic('first local call'); end if;
          if Work() <> 42 then panic('shared local cell'); end if;
        end.",
    );
}

#[test]
fn unknown_helper_results_can_transfer_immutable_closures() {
    assert_succeeds(
        "program UnknownSafeFactory;
        uses Std.Tasks as Tasks;
        function Identity(Work: function(): integer): function(): integer;
        begin return Work; end function;
        begin scope
          const Value := 42;
          const Work := Identity(function(): integer begin return Value; end function);
          const Job := go Work();
          if Tasks.Wait(Job) <> 42 then panic('immutable helper result'); end if;
        end scope; end.",
    );
}

#[test]
fn unknown_task_arguments_and_channel_values_retain_runtime_guards() {
    for boundary in [
        "scope const Job := go Invoke(Work); discard Tasks.Wait(Job); end scope;",
        "const Queue: channel of function(): integer := Tasks.CreateChannel(1);
         case Tasks.Send(Queue, Work) of
           when Ok(_): begin end;
           when Error(const Cause): panic(Cause.Message());
         end case;",
    ] {
        let source = format!(
            "program UnknownBoundary;
            uses Std.Tasks as Tasks;
            function Invoke(Work: function(): integer): integer;
            begin return Work(); end function;
            procedure Transfer(Work: function(): integer);
            begin {boundary} end procedure;
            function Make(): function(): integer;
            begin
              var Value := 42;
              return function(): integer begin return Value; end function;
            end function;
            begin Transfer(Make()); end."
        );
        let error = run_program(&source).expect_err("unknown local graph cannot transfer");
        assert!(error.message.contains("task-bound"), "{error:?}");
    }
}

#[test]
fn unknown_task_result_is_checked_when_published() {
    let error = run_program(
        "program UnknownTaskResult;
        uses Std.Tasks as Tasks;
        function Make(): function(): integer;
        begin
          var Value := 42;
          return function(): integer begin return Value; end function;
        end function;
        function Worker(Factory: function(): function(): integer): function(): integer;
        begin return Factory(); end function;
        begin scope
          const Job := go Worker(Make);
          discard Tasks.Wait(Job);
        end scope; end.",
    )
    .expect_err("unknown local result cannot leave its worker");
    assert!(error.message.contains("task-bound"), "{error:?}");
}

#[test]
fn known_local_task_results_are_diagnosed_before_lowering() {
    let source = "program KnownTaskResult;
        function Make(): function(): integer;
        begin
          var Value := 42;
          return function(): integer begin return Value; end function;
        end function;
        begin scope const Job := go Make(); discard Job; end scope; end.";
    let errors = crate::compile(&parse_ok(source)).expect_err("known local task result");
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE),
        "{errors:#?}"
    );
}
