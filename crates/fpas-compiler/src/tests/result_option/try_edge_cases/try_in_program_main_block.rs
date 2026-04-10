use super::super::{compile_and_run, compile_err};
// ── Try in program main block ───────────────────────────────────────────

#[test]
fn try_result_in_program_main_block_is_compile_error() {
    let err = compile_err(
        "program T;
function GetVal(): Result of integer, string;
begin
  return Ok(42)
end;
begin
  var X: integer := try GetVal()
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn try_option_in_program_main_block_is_compile_error() {
    let err = compile_err(
        "program T;
function MaybeVal(): Option of integer;
begin
  return Some(7)
end;
begin
  var X: integer := try MaybeVal()
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}
