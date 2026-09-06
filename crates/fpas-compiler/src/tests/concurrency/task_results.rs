//! Regression coverage for independently typed task results.

use super::*;

#[test]
fn wait_preserves_integer_before_procedure_task_results() {
    assert_succeeds(
        "program Reversed; uses Std.Task; function Number(): integer; begin return 7 end; procedure Work(); begin end; begin if Wait(go Number()) <> 7 then panic('wrong result'); Wait(go Work()) end.",
    );
}

#[test]
fn wait_preserves_mixed_direct_spawn_results() {
    assert_succeeds(
        "program MixedDirect; uses Std.Task; function Number(): integer; begin return 7 end; procedure Work(); begin end; begin Wait(go Work()); if Wait(go Number()) <> 7 then panic('wrong result') end.",
    );
}

#[test]
fn wait_preserves_mixed_procedure_and_integer_task_results() {
    assert_succeeds(
        r#"program MixedTaskResults;
uses Std.Task;
function Number(): integer;
begin
  return 7
end;
procedure Work();
begin
end;
begin
  var A: task := go Number();
  var B: task := go Work();
  Wait(B);
  if Wait(A) <> 7 then panic('wrong task result')
end."#,
    );
}

#[test]
fn wait_preserves_mixed_results_across_routines_and_loop_branches() {
    assert_succeeds(
        r#"program MixedTaskRoutines;
uses Std.Task;
function Number(): integer;
begin
  return 7
end;
procedure Work();
begin
end;
procedure WaitForWork();
begin
  Wait(go Work());
  for Index: integer := 1 to 3 do
  begin
    Wait(go Work());
    if Index < 1 then panic('wrong loop branch')
  end
end;
function WaitForNumber(): integer;
begin
  return Wait(go Number())
end;
begin
  WaitForWork();
  if WaitForNumber() <> 7 then panic('wrong task result')
end."#,
    );
}

#[test]
fn wait_signature_merges_unit_and_value_without_erasing_call_types() {
    let ast = parse_ok(
        "program MixedSignature; uses Std.Task; function Number(): integer; begin return 7 end; procedure Work(); begin end; begin Wait(go Work()); if Wait(go Number()) <> 7 then panic('wrong result') end.",
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
        "program UnitSignature; uses Std.Task; procedure Work(); begin end; begin Wait(go Work()); Wait(go Work()) end.",
    );
    let ir = crate::lower(&ast).expect("procedure task result IR");
    let wait = fpas_ir::IntrinsicId::new(u32::from(u16::from(fpas_bytecode::Intrinsic::Task(
        fpas_bytecode::TaskIntrinsic::Wait,
    ))));
    let signature = ir.intrinsic(wait).expect("Wait signature");
    assert_eq!(ir.ty(signature.result).unwrap().kind, fpas_ir::IrType::Unit);
}
