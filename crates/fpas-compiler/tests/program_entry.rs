//! Program-entry identities must not occupy the source declaration namespace.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`

use fpas_bytecode::VerifiedExecutable;
use fpas_vm::{
    DebugRecordingEnvelope, DebugRunResult, DebugSession, DebugStopReason, FunctionBreakpoint,
};

fn parse(source: &str) -> fpas_parser::Program {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:#?}");
    program
}

fn linked(source: &str) -> VerifiedExecutable {
    let object = fpas_compiler::compile_program_object_with_support(&parse(source), &[], &[])
        .unwrap_or_else(|errors| panic!("{errors:#?}"));
    fpas_linker::link_objects(&[], &object).expect("link program")
}

fn run_both_paths(source: &str) {
    let executable =
        fpas_compiler::compile(&parse(source)).unwrap_or_else(|errors| panic!("{errors:#?}"));
    fpas_vm::Vm::new(executable)
        .run()
        .expect("direct execution");
    fpas_vm::Vm::new(linked(source))
        .run()
        .expect("linked execution");
    let object = fpas_compiler::compile_object(&parse(source)).expect("standalone object");
    let executable = fpas_linker::link_objects(&[], &object).expect("link standalone object");
    fpas_vm::Vm::new(executable)
        .run()
        .expect("standalone object execution");
}

#[test]
fn program_and_function_names_can_match_in_any_case() {
    for name in ["Greet", "greet", "GREET"] {
        run_both_paths(&format!(
            "program Greet;
             function {name}(N: integer): integer;
             begin return N + 1; end function;
             begin if Greet(41) <> 42 then panic('call'); end if; end program;"
        ));
    }
}

#[test]
fn program_and_procedure_names_can_match() {
    run_both_paths(
        "program Touch;
        mutable var Value: integer := 0;
        procedure touch(); begin Value := 42; end procedure;
        begin TOUCH(); if Value <> 42 then panic('procedure'); end if; end program;",
    );
}

#[test]
fn matching_routine_keeps_recursion_and_function_values() {
    run_both_paths(
        "program Count;
        function Count(N: integer): integer;
        begin if N = 0 then return 0; else return Count(N - 1) + 1; end if; end function;
        begin var F: function(N: integer): integer := Count;
          if F(4) <> 4 then panic('recursive function value'); end if;
        end program;",
    );
}

#[test]
fn matching_routine_keeps_closure_captures() {
    run_both_paths(
        "program Counter;
        function Counter(): function(): integer;
        begin
          mutable var Value: integer := 0;
          return function(): integer
          begin Value := Value + 1; return Value; end function;
        end function;
        begin var Next: function(): integer := Counter();
          if Next() <> 1 then panic('first capture'); end if;
          if Next() <> 2 then panic('second capture'); end if;
        end program;",
    );
}

#[test]
fn matching_routine_keeps_nested_routine_resolution() {
    run_both_paths(
        "program MakeAdder;
        function MakeAdder(Base: integer): function(Value: integer): integer;
          function Add(Value: integer): integer;
          begin return Base + Value; end function;
        begin return Add; end function;
        begin var AddForty: function(Value: integer): integer := MakeAdder(40);
          if AddForty(2) <> 42 then panic('nested routine'); end if;
        end program;",
    );
}

#[test]
fn program_name_can_match_a_global_or_type() {
    run_both_paths(
        "program Value;
        mutable var Value: integer := 42;
        begin if Value <> 42 then panic('global'); end if; end program;",
    );
    run_both_paths(
        "program Model;\n\ntype Model = record\n  Value: integer;\nend record;\n\nbegin\n  var Item: Model := Model(Value := 42);\n  if Item.Value <> 42 then\n    panic('type');\n  end if;\nend program;\n",
    );
}

#[test]
fn unit_initializer_and_same_named_routines_keep_distinct_link_identities() {
    let (unit, errors) = fpas_parser::parse_compilation_unit(
        "unit Greet;
        public function Greet(): integer;
        begin return 41; end function;
        public var Seed: integer := Greet();
        end unit;",
    );
    assert!(errors.is_empty(), "{errors:#?}");
    let fpas_parser::CompilationUnit::Unit(unit) = unit else {
        panic!("expected unit");
    };
    let unit = fpas_compiler::compile_unit_object(&unit, &[]).expect("compile unit");
    let program = parse(
        "program Greet; uses Greet as Helpers;
        function Greet(): integer; begin return Helpers.Seed + 1; end function;
        begin if Greet() <> 42 then panic('linked names'); end if; end program;",
    );
    let object =
        fpas_compiler::compile_program_object_with_support(&program, &[unit.interface], &[])
            .expect("compile program with unit");
    let executable = fpas_linker::link_objects(&[unit.object], &object).expect("link unit");
    let entry = executable.executable().entry;
    let mut session = DebugSession::new(executable.clone()).expect("unit debug session");
    let bound = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "GREET".into(),
        }])
        .expect("bind both declared routines");
    assert_eq!(bound[0].functions.len(), 2);
    assert!(!bound[0].functions.contains(&entry));
    let qualified = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "Greet.Greet".into(),
        }])
        .expect("bind qualified unit routine");
    assert_eq!(qualified[0].functions.len(), 1);
    assert!(bound[0].functions.contains(&qualified[0].functions[0]));
    let DebugRunResult::Stopped(stop) =
        session.continue_execution().expect("unit initializer call")
    else {
        panic!("expected unit routine breakpoint during initialization");
    };
    assert_eq!(stop.reason, DebugStopReason::Breakpoint);
    let stack = session.stack(0, 10).expect("initializer stack");
    assert_eq!(stack.items.len(), 3);
    assert_eq!(stack.items[0].name, "greet.greet");
    assert!(stack.items[1..].iter().all(|frame| frame.name == "greet"));
    session
        .replace_function_breakpoints(Vec::new())
        .expect("clear unit breakpoint");
    assert!(matches!(
        session.continue_execution().expect("finish unit debuggee"),
        DebugRunResult::Terminated(_)
    ));
    fpas_vm::Vm::new(executable)
        .run()
        .expect("execute with unit");
}

#[test]
fn actual_duplicate_routines_still_report_a_semantic_error() {
    let program = parse(
        "program Greet;
        function Greet(): integer; begin return 1; end function;
        function gReEt(): integer; begin return 2; end function;
        begin null; end program;",
    );
    let errors = fpas_compiler::compile_object(&program).expect_err("duplicate routines");
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "{errors:#?}"
    );
    assert!(
        errors.iter().all(|error| error.code.value() < 9000),
        "{errors:#?}"
    );
}

#[test]
fn debugger_preserves_program_display_and_breaks_only_in_declared_routine() {
    let executable = linked(
        "program Greet;
function Greet(N: integer): integer;
begin
  return N + 1;
end function;
begin
  var Answer: integer := Greet(41);
  if Answer <> 42 then panic('answer'); end if;
end program;",
    );
    assert_eq!(
        DebugRecordingEnvelope::from_executable(&executable)
            .expect("recording identity")
            .program,
        "greet"
    );
    let entry = executable.executable().entry;
    let mut session = DebugSession::new(executable).expect("debug session");
    let bound = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "GREET".into(),
        }])
        .expect("bind routine");
    assert_eq!(bound[0].functions.len(), 1);
    assert_ne!(bound[0].functions[0], entry);
    let DebugRunResult::Stopped(stop) = session.continue_execution().expect("routine stop") else {
        panic!("expected routine breakpoint");
    };
    assert_eq!(stop.reason, DebugStopReason::Breakpoint);
    assert_eq!(stop.breakpoint_ids, vec![bound[0].id]);
    assert_eq!(stop.location.expect("routine location").line, 4);
    let stack = session.stack(0, 10).expect("stack");
    assert_eq!(stack.items.len(), 2);
    assert!(stack.items.iter().all(|frame| frame.name == "greet"));
    session
        .replace_function_breakpoints(Vec::new())
        .expect("clear breakpoint");
    assert!(matches!(
        session.continue_execution().expect("finish"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn program_name_alone_does_not_bind_a_routine_breakpoint() {
    let mut session = DebugSession::new(linked("program OnlyProgram; begin null; end program;"))
        .expect("debug session");
    let bound = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "ONLYPROGRAM".into(),
        }])
        .expect("bind missing routine");
    assert!(bound[0].functions.is_empty());
    assert!(!bound[0].is_verified());
}
