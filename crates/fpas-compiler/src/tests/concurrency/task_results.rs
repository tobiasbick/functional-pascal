//! Regression coverage for independently typed task results.

use super::*;

#[test]
fn standard_intrinsic_tasks_preserve_result_and_qualified_dispatch() {
    assert_succeeds(
        "program IntrinsicTasks; uses Std.Tasks, Std.Str, Std.Math; begin if Wait(go Std.Str.Length('abc')) <> 3 then panic('qualified result'); end if; if Wait(go Abs(-7)) <> 7 then panic('short result'); end if; if Wait(go Std.Str.Format('%d:%s', 4, 'x')) <> '4:x' then panic('variadic result'); end if; end.",
    );
}

#[test]
fn intrinsic_procedure_tasks_support_empty_and_variadic_output() {
    assert_succeeds(
        "program IntrinsicProcedures; uses Std.Tasks, Std.Console; begin Wait(go WriteLn()); Wait(go Std.Console.WriteLn('value:', 7)); end.",
    );
}

#[test]
fn source_routine_shadowing_does_not_become_an_intrinsic_task() {
    assert_succeeds(
        "program ShadowTask; uses Std.Tasks; function Abs(Value: integer): integer; begin return Value + 10; end function; begin if Wait(go Abs(-7)) <> 3 then panic('shadowed task'); end if; end.",
    );
}

#[test]
fn wait_preserves_integer_before_procedure_task_results() {
    assert_succeeds(
        "program Reversed; uses Std.Tasks; function Number(): integer; begin return 7; end function; procedure Work(); begin end procedure; begin if Wait(go Number()) <> 7 then panic('wrong result'); end if; Wait(go Work()); end.",
    );
}

#[test]
fn wait_preserves_mixed_direct_spawn_results() {
    assert_succeeds(
        "program MixedDirect; uses Std.Tasks; function Number(): integer; begin return 7; end function; procedure Work(); begin end procedure; begin Wait(go Work()); if Wait(go Number()) <> 7 then panic('wrong result'); end if; end.",
    );
}

#[test]
fn wait_preserves_mixed_procedure_and_integer_task_results() {
    assert_succeeds(
        r#"program MixedTaskResults;
uses Std.Tasks;
function Number(): integer;
begin
  return 7;
end function;
procedure Work();
begin
end procedure;
begin
  var A: task := go Number();
  var B: task := go Work();
  Wait(B);
  if Wait(A) <> 7 then panic('wrong task result'); end if;
end."#,
    );
}

#[test]
fn wait_preserves_mixed_results_across_routines_and_loop_branches() {
    assert_succeeds(
        r#"program MixedTaskRoutines;
uses Std.Tasks;
function Number(): integer;
begin
  return 7;
end function;
procedure Work();
begin
end procedure;
procedure WaitForWork();
begin
  Wait(go Work());
  for Index: integer := 1 to 3 do
  begin
    Wait(go Work());
    if Index < 1 then panic('wrong loop branch'); end if;
  end; end for;
end procedure;
function WaitForNumber(): integer;
begin
  return Wait(go Number());
end function;
begin
  WaitForWork();
  if WaitForNumber() <> 7 then panic('wrong task result'); end if;
end."#,
    );
}

#[test]
fn wait_signature_merges_unit_and_value_without_erasing_call_types() {
    let ast = parse_ok(
        "program MixedSignature; uses Std.Tasks; function Number(): integer; begin return 7; end function; procedure Work(); begin end procedure; begin Wait(go Work()); if Wait(go Number()) <> 7 then panic('wrong result'); end if; end.",
    );
    let ir = crate::lower(&ast).expect("mixed task result IR");
    let wait = fpas_ir::IntrinsicId::new(u32::from(u16::from(fpas_bytecode::Intrinsic::Task(
        fpas_bytecode::TaskIntrinsic::Wait,
    ))));
    let signature = ir.intrinsic(wait).expect("Wait signature");
    assert_eq!(
        ir.ty(signature.result).unwrap().kind,
        fpas_ir::IrType::Dynamic
    );
    let results: Vec<_> = ir.functions.iter().flat_map(|function| &function.blocks)
        .flat_map(|block| &block.instructions)
        .filter(|instruction| matches!(instruction.operation, fpas_ir::Operation::Intrinsic { intrinsic, .. } if intrinsic == wait))
        .map(|instruction| &ir.ty(instruction.result.unwrap().ty).unwrap().kind)
        .collect();
    assert_eq!(results, [&fpas_ir::IrType::Unit, &fpas_ir::IrType::Integer]);
}

#[test]
fn wait_signature_keeps_unit_for_procedure_only_calls() {
    let ast = parse_ok(
        "program UnitSignature; uses Std.Tasks; procedure Work(); begin end procedure; begin Wait(go Work()); Wait(go Work()); end.",
    );
    let ir = crate::lower(&ast).expect("procedure task result IR");
    let wait = fpas_ir::IntrinsicId::new(u32::from(u16::from(fpas_bytecode::Intrinsic::Task(
        fpas_bytecode::TaskIntrinsic::Wait,
    ))));
    let signature = ir.intrinsic(wait).expect("Wait signature");
    assert_eq!(ir.ty(signature.result).unwrap().kind, fpas_ir::IrType::Unit);
}
