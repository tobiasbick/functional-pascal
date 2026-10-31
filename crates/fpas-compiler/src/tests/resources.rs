//! Deterministic release through local and linked record methods.

use super::{assert_succeeds, compiled_unit, parse_ok, run_program};

#[test]
fn resource_release_precedes_postconditions_and_failure_skips_them() {
    let source = "program T;
var Released: integer := 0;
type R = record release procedure Close(Self: R); begin Released := Released + 1; end procedure; end record;
function Work(var Observed: integer): integer; ensures Observed = 1;
begin using const Item := R(); return 7; end function;
begin if Work(var Released) <> 7 then panic('return value'); end if; end.";
    assert_succeeds(source);
    let failing = source.replace("Released := Released + 1;", "panic('release failed');");
    let error = run_program(&failing).expect_err("release failure stops the return");
    assert!(error.message.contains("release failed"), "{error:?}");
    assert!(!error.message.contains("ensures Observed"), "{error:?}");
}

#[test]
fn anonymous_routine_releases_its_resource_on_return() {
    assert_succeeds(
        "program T; var Released: integer := 0;
type R = record release procedure Close(Self: R); begin Released := Released + 1; end procedure; end record;
begin const Work: function(): integer := function(): integer begin using const Item := R(); return 7; end function;
if Work() <> 7 then panic('return value'); end if;
if Released <> 1 then panic('resource not released'); end if; end.",
    );
}

#[test]
fn conditional_existing_resource_keeps_its_publication_boundary() {
    assert_succeeds(
        "program T; var Released: integer := 0; type R = record release procedure Close(Self: R); begin Released := Released + 1; end procedure; end record; function Choose(): boolean; begin return true; end function; begin const Existing := R(); begin using const Owned := if Choose() then Existing else Existing end if; end; if Released <> 1 then panic('conditional alias release'); end if; end.",
    );
}

#[test]
fn custom_resources_release_in_reverse_order() {
    assert_succeeds(
        "program T; var Seen: integer := 0; type R = record Value: integer; release procedure Close(Self: R); begin Seen := Seen * 10 + Self.Value; end procedure; end record; begin begin using const A := R(Value := 1); using const B := R(Value := 2); end; if Seen <> 21 then panic('release order'); end if; end.",
    );
}

#[test]
fn wrapped_and_branching_resource_initializers_release_the_selected_value() {
    assert_succeeds(
        "program T;
var Seen: integer := 0;
type R = record Value: integer; release procedure Close(Self: R); begin Seen := Seen * 10 + Self.Value; end procedure; end record;
function Acquire(Value: integer): result of (R, string); begin return Ok(R(Value := Value)); end function;
function Convert(Message: string): string; begin return Message; end function;
function Run(Choose: boolean): result of (integer, string);
begin
  begin using const A := Acquire(1).Unwrap(); end;
  begin using const B := try Acquire(2).MapError(Convert); end;
  begin using const C := if Choose then Acquire(3).Unwrap() else Acquire(9).Unwrap() end if; end;
  begin using const D := case Choose of when true: Acquire(4).Unwrap(); when false: Acquire(9).Unwrap(); end case; end;
  return Ok(Seen);
end function;
begin if Run(true).Unwrap() <> 1234 then panic('wrong resources'); end if; end.",
    );
}

#[test]
fn wrapper_and_branch_acquisitions_arm_the_factory_call() {
    use fpas_code::Opcode;

    let source = "program T;
type R = record release procedure Close(Self: R); begin null; end procedure; end record;
function Acquire(): result of (R, string); begin return Ok(R()); end function;
function Convert(Message: string): string; begin return Message; end function;
function Work(Choose: boolean): result of (integer, string);
begin
  begin using const A := Acquire().Unwrap(); end;
  begin using const B := try Acquire().MapError(Convert); end;
  begin using const C := if Choose then Acquire().Unwrap() else Acquire().Unwrap() end if; end;
  begin using const D := case Choose of when true: Acquire().Unwrap(); when false: Acquire().Unwrap(); end case; end;
  return Ok(1);
end function;
begin discard Work(true); end.";
    let image = crate::compile(&parse_ok(source)).expect("compile resource fixture");
    let executable = image.executable();
    let factory = executable
        .functions
        .iter()
        .position(|function| {
            executable.strings.get(function.name).is_some_and(|name| {
                name.eq_ignore_ascii_case("T.Acquire") || name.eq_ignore_ascii_case("Acquire")
            })
        })
        .expect("factory function");
    let calls: Vec<_> = executable
        .code
        .iter()
        .enumerate()
        .filter(|(_, instruction)| {
            instruction.opcode() == Ok(Opcode::CallDirect)
                && instruction
                    .abc_operands()
                    .is_ok_and(|operands| usize::from(operands.b) == factory)
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(calls.len(), 6);
    for call in calls {
        assert_eq!(
            executable.code[call - 1].opcode(),
            Ok(Opcode::ArmResourceInit)
        );
    }
}

#[test]
fn generic_record_release_is_instantiated() {
    assert_succeeds(
        "program T; var Count: integer := 0; type Box of T = record Value: T; release procedure Close(Self: Box of T); begin Count := Count + 1; end procedure; end record; begin begin using const B: Box of integer := Box(Value := 1); end; if Count <> 1 then panic('generic release'); end if; end.",
    );
}

#[test]
fn synchronous_callbacks_borrow_resource_until_release() {
    assert_succeeds(
        "program T; var Count: integer := 0; type R = record release procedure Close(Self: R); begin Count := Count + 10; end procedure; end record; procedure Borrow(Value: R); begin Count := Count + 1; end procedure; begin begin using const Value := R(); const Callback: procedure() := procedure() begin Borrow(Value); end procedure; Callback(); [Value].ForEach(procedure(Item: R) begin Borrow(Item); end procedure); end; if Count <> 12 then panic('resource callback'); end if; end.",
    );
}

#[test]
fn loop_control_releases_each_implicit_body_scope() {
    assert_succeeds(
        "program T; var Seen: integer := 0; type R = record Value: integer; release procedure Close(Self: R); begin Seen := Seen * 10 + Self.Value; end procedure; end record; begin for I: integer := 1 to 3 do using const Item := R(Value := I); if I = 2 then continue; end if; if I = 3 then break; end if; end for; if Seen <> 123 then panic('loop release'); end if; end.",
    );
}

#[test]
fn imported_private_release_is_linked() {
    let dependency = compiled_unit(
        "unit Demo.Resources; var Count: integer := 0; public type R = record public Value: integer; release procedure Close(Self: R); begin Count := Count + 1; end procedure; end record; public function Make(): R; begin return R(Value := 1); end function; public function Released(): integer; begin return Count; end function; end unit;",
        &[],
    );
    let interfaces = [dependency.interface.clone()];
    let access = parse_ok(
        "program T; uses Demo.Resources as API; begin const Value := API.Make(); Value.Close(); end.",
    );
    let errors = crate::compile_program_object_with_support(&access, &interfaces, &interfaces)
        .expect_err("release method must remain source-private");
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_PRIVATE_RECORD_MEMBER),
        "{errors:?}"
    );
    let program = parse_ok(
        "program T; uses Demo.Resources as API; begin begin using const Value := API.Make(); end; if API.Released() <> 1 then panic('imported release'); end if; end.",
    );
    let root = crate::compile_program_object_with_support(&program, &interfaces, &interfaces)
        .expect("consumer object");
    let executable = fpas_linker::link_objects(&[dependency.object], &root).expect("link");
    fpas_runtime::execution::Vm::new(executable)
        .run()
        .expect("execute");
}

#[test]
fn imported_generic_private_release_is_linked() {
    let dependency = compiled_unit(
        "unit Demo.Resources; var Count: integer := 0; public type R of T = record public Value: T; release procedure Close(Self: R of T); begin Count := Count + 1; end procedure; end record; public function Make(): R of integer; begin return R(Value := 1); end function; public function Released(): integer; begin return Count; end function; end unit;",
        &[],
    );
    let interfaces = [dependency.interface.clone()];
    let program = parse_ok(
        "program T; uses Demo.Resources as API; begin begin using const Value := API.Make(); end; if API.Released() <> 1 then panic('imported generic release'); end if; end.",
    );
    let root = crate::compile_program_object_with_support(&program, &interfaces, &interfaces)
        .expect("consumer object");
    let executable = fpas_linker::link_objects(&[dependency.object], &root).expect("link");
    fpas_runtime::execution::Vm::new(executable)
        .run()
        .expect("execute");
}

#[test]
fn verifier_rejects_invalid_resource_lifetime_control_flow() {
    use fpas_code::{Instruction, Opcode, ValidationErrorKind};

    let source = format!(
        "program T; var Flag: boolean := true; var Seen: integer := 0; type R = record release procedure Close(Self: R); begin null; end procedure; end record; begin begin using const Value := R(); if Flag then Seen := 1; else Seen := 2; end if; end; {} end.",
        "Seen := Seen + 1; ".repeat(20)
    );
    let original = crate::compile(&parse_ok(&source))
        .expect("compile resource fixture")
        .into_unverified();
    let entry = &original.functions[usize::from(original.entry.get())];
    let start = entry.code.start.get() as usize;
    let end = entry.code.end.get() as usize;
    let find = |opcode| {
        (start..end)
            .find(|&i| original.code[i].opcode() == Ok(opcode))
            .expect("fixture opcode")
    };
    let enter = find(Opcode::EnterCleanupScope);
    let register = find(Opcode::RegisterCleanup);
    let exit = find(Opcode::ExitCleanupScope);
    let ret = find(Opcode::Return);
    let verify = |candidate: fpas_code::Executable, expected: &str| {
        let error = candidate
            .verify()
            .expect_err("invalid resource lifetime must be rejected");
        assert!(
            matches!(error.kind, ValidationErrorKind::CleanupState { reason } if reason.contains(expected)),
            "{error:?}"
        );
    };

    let mut outside = original.clone();
    outside.code[enter] = outside.code[register];
    verify(outside, "resource publication");

    let mut unguarded = original.clone();
    unguarded.code[enter] = Instruction::abc(Opcode::ArmResourceInit, 0, 0, 0, 0).expect("arm");
    verify(unguarded, "no initializer guard");

    let mut nested_initializer = original.clone();
    nested_initializer.code[register] =
        Instruction::abc(Opcode::BeginResourceInit, 0, 0, 0, 0).expect("begin initializer");
    verify(nested_initializer, "initializer requires");

    let mut unmatched = original.clone();
    unmatched.code[enter] =
        Instruction::abc(Opcode::ExitCleanupScope, u16::MAX, 0, 0, 0).expect("exit");
    verify(unmatched, "resource lifetime exit");

    let mut crossed = original.clone();
    crossed.code[exit] = Instruction::abc(Opcode::CancelTaskScope, 0, 0, 0, 0).expect("task exit");
    verify(crossed, "task scope exit");

    let mut abandoned = original.clone();
    abandoned.code[exit] = abandoned.code[ret];
    verify(abandoned, "abandons a live resource lifetime");

    let close = original
        .functions
        .iter()
        .position(|function| {
            original
                .strings
                .get(function.name)
                .is_some_and(|name| name.to_ascii_lowercase().ends_with("r.close"))
        })
        .expect("release routine");
    let resource = original.code[register]
        .abc_operands()
        .expect("resource operands")
        .a;
    let mut tail = original.clone();
    tail.code[exit] =
        Instruction::abc(Opcode::TailCall, u16::MAX, close as u16, resource, 1).expect("tail call");
    tail.code[exit + 1] = tail.code[ret];
    verify(tail, "abandons a live resource lifetime");

    let branch = (register + 1..exit)
        .find(|&i| {
            matches!(
                original.code[i].opcode(),
                Ok(Opcode::BranchIfFalse | Opcode::BranchIfTrue)
            )
        })
        .expect("conditional inside resource scope");
    let opcode = original.code[branch].opcode().expect("branch opcode");
    let condition = original.code[branch]
        .abx_operands()
        .expect("branch operands")
        .a;
    let mut merge = original.clone();
    merge.code[branch] = Instruction::abx(opcode, condition, (exit + 1) as u32).expect("branch");
    verify(merge, "control-flow merge");
}

#[test]
fn verifier_rejects_unprotected_acquisition_on_any_branch() {
    use fpas_code::{Instruction, Opcode, ValidationErrorKind};
    let source = "program T; var Flag: boolean := true; type R = record release procedure Close(Self: R); begin null; end procedure; end record; function Acquire(): R; begin return R(); end function; begin using const Item := if Flag then Acquire() else Acquire() end if; end.";
    let original = crate::compile(&parse_ok(source))
        .expect("compile fixture")
        .into_unverified();
    let factory = original
        .functions
        .iter()
        .position(|function| {
            original.strings.get(function.name).is_some_and(|name| {
                name.eq_ignore_ascii_case("T.Acquire") || name.eq_ignore_ascii_case("Acquire")
            })
        })
        .expect("factory");
    let arms: Vec<_> = original
        .code
        .iter()
        .enumerate()
        .filter(|(index, instruction)| {
            *index > 0
                && instruction.opcode() == Ok(Opcode::CallDirect)
                && instruction
                    .abc_operands()
                    .is_ok_and(|operands| usize::from(operands.b) == factory)
                && original.code[*index - 1].opcode() == Ok(Opcode::ArmResourceInit)
        })
        .map(|(index, _)| index - 1)
        .collect();
    assert_eq!(arms.len(), 2, "each branch arms the factory");
    for arm in arms {
        let mut corrupted = original.clone();
        corrupted.code[arm] = Instruction::abc(Opcode::Move, 0, 0, 0, 0).expect("move");
        let error = corrupted
            .verify()
            .expect_err("unprotected call must be rejected");
        assert!(
            matches!(error.kind, ValidationErrorKind::CleanupState { reason } if reason.contains("unprotected call")),
            "{error:?}"
        );
    }
}

#[test]
fn verifier_allows_unarmed_publication_of_an_existing_value() {
    use fpas_code::{Instruction, Opcode};
    let source = "program T; type R = record release procedure Close(Self: R); begin null; end procedure; end record; begin const Existing := R(); begin using const Alias := Existing; end; end.";
    let mut candidate = crate::compile(&parse_ok(source))
        .expect("compile fixture")
        .into_unverified();
    let arms: Vec<_> = candidate
        .code
        .iter()
        .enumerate()
        .filter_map(|(index, instruction)| {
            (instruction.opcode() == Ok(Opcode::ArmResourceInit)).then_some(index)
        })
        .collect();
    assert_eq!(arms.len(), 1, "fixture has one publication handoff");
    candidate.code[arms[0]] = Instruction::abc(Opcode::Move, 0, 0, 0, 0).expect("move");
    candidate
        .verify()
        .expect("an existing value needs no acquisition handoff");
}
