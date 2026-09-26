use super::*;

#[test]
fn identical_source_produces_deterministic_ir_and_bytecode() {
    let source = "\
program RegisterDeterministic;
begin
  mutable var X: integer := 1;
  if X = 1 then X := X + 2;
  if X <> 3 then panic('bad')
end.";
    let program = parse_ok(source);

    let first_ir = crate::lower(&program).expect("first lowering should succeed");
    let second_ir = crate::lower(&program).expect("second lowering should succeed");
    assert_eq!(first_ir, second_ir);

    let first = crate::compile(&program).expect("first compile should succeed");
    let second = crate::compile(&program).expect("second compile should succeed");
    assert_eq!(first, second);
}

#[test]
fn object_compilation_is_deterministic_and_linkable() {
    let program = parse_ok("program Demo; begin end.");
    let first = crate::compile_object(&program).expect("first object");
    let second = crate::compile_object(&program).expect("second object");
    assert_eq!(
        fpas_unit::object::encode_object(&first).expect("first bytes"),
        fpas_unit::object::encode_object(&second).expect("second bytes")
    );
    let linked = fpas_linker::link_objects(&[], &first).expect("register link");
    assert_eq!(linked.executable().entry, fpas_bytecode::FunctionId::new(0));
}

#[test]
fn small_program_has_register_style_instruction_count() {
    let program = parse_ok(
        "\
program RegisterCount;
begin
  var A: integer := 1;
  mutable var B: integer := 2;
  B := A + B
end.",
    );
    let executable = crate::compile(&program).expect("compiler compilation should succeed");
    let image = executable.executable();

    assert_eq!(image.code.len(), 8);
    let add = image
        .code
        .iter()
        .find(|instruction| instruction.opcode() == Ok(fpas_bytecode::Opcode::AddInteger))
        .expect("assignment should contain integer addition")
        .abc_operands()
        .expect("integer addition uses ABC operands");
    assert_eq!(add.a, 1, "addition should write directly into local B");
    assert_eq!(
        image
            .code
            .iter()
            .filter(|instruction| {
                instruction.opcode() == Ok(fpas_bytecode::Opcode::Move)
                    && instruction
                        .abc_operands()
                        .is_ok_and(|operands| operands.a == 1)
            })
            .count(),
        1,
        "only B's exact source initializer should move into local B"
    );
    assert!(image.functions[0].register_count <= 5);
}

#[test]
fn repeated_temporaries_reuse_the_lowest_free_registers() {
    let program = parse_ok(
        "\
program RegisterReuse;
begin
  mutable var X: integer := 0;
  X := 1 + 2;
  X := 3 + 4;
  X := 5 + 6;
  X := 7 + 8;
  if X <> 15 then panic('bad')
end.",
    );
    let executable = crate::compile(&program).expect("compiler compilation should succeed");

    assert!(executable.executable().functions[0].register_count <= 4);
}

#[test]
fn every_emitted_register_operand_passes_verifier_admission() {
    let program = parse_ok(
        "\
program RegisterVerified;
begin
  mutable var X: integer := 0;
  while X < 10 do X := X + 1
end.",
    );
    let verified =
        crate::compile(&program).expect("compiler must return only a verified executable");
    let candidate = verified.into_unverified();

    candidate
        .verify()
        .expect("all generated operands should verify");
}

#[test]
fn integer_loops_emit_fused_comparison_and_for_loop() {
    let program = parse_ok(
        "program FusedLoop; begin mutable var Total: integer := 0; for I: integer := 1 to 4 do Total := Total + I; if Total <> 10 then panic('wrong') end.",
    );
    let executable = crate::compile(&program).expect("loop compiles");
    let code = &executable.executable().code;
    assert!(
        code.iter()
            .any(|word| word.opcode() == Ok(fpas_bytecode::Opcode::ForLoop))
    );
    assert!(
        code.iter()
            .any(|word| word.opcode() == Ok(fpas_bytecode::Opcode::BranchIfLessEqualInteger))
    );
    assert!(
        code.iter()
            .any(|word| word.opcode() == Ok(fpas_bytecode::Opcode::BranchIfNotEqualInteger))
    );
}

#[test]
fn single_use_integer_literals_emit_immediate_operations() {
    let source = "program ImmediateInteger; begin mutable var X: integer := 5; X := X + 7; X := X div 3; X := X + (-3); if X <> 1 then panic('wrong') end.";
    let program = parse_ok(source);
    let executable = crate::compile(&program).expect("immediate arithmetic compiles");
    let code = &executable.executable().code;
    assert!(
        code.iter()
            .any(|word| word.opcode() == Ok(fpas_bytecode::Opcode::AddIntegerImm))
    );
    assert!(
        code.iter()
            .any(|word| word.opcode() == Ok(fpas_bytecode::Opcode::DivideIntegerImm))
    );
    assert_succeeds(source);
}

#[test]
fn string_ordering_uses_typed_opcodes() {
    let source = "program StringOrdering; begin if not ('a' < 'b') then panic('less'); if not ('b' > 'a') then panic('greater'); if not ('a' <= 'a') then panic('less equal'); if not ('b' >= 'b') then panic('greater equal') end.";
    let program = parse_ok(source);
    let executable = crate::compile(&program).expect("string ordering compiles");
    let code = &executable.executable().code;
    for opcode in [
        fpas_bytecode::Opcode::LessString,
        fpas_bytecode::Opcode::GreaterString,
        fpas_bytecode::Opcode::LessEqualString,
        fpas_bytecode::Opcode::GreaterEqualString,
    ] {
        assert!(code.iter().any(|word| word.opcode() == Ok(opcode)));
    }
    assert_succeeds(source);
}

#[test]
fn string_append_consumes_dead_left_temporaries() {
    let source = "program Append; begin mutable var S: string := ''; for I: integer := 1 to 3 do S := S + 'x'; var T: string := ('a' + S) + 'b'; if T <> 'axxxb' then panic('wrong') end.";
    let program = parse_ok(source);
    let executable = crate::compile(&program).expect("string append compiles");
    let concatenations = executable
        .executable()
        .code
        .iter()
        .filter(|word| word.opcode() == Ok(fpas_bytecode::Opcode::ConcatString))
        .collect::<Vec<_>>();
    assert!(!concatenations.is_empty());
    assert!(
        concatenations
            .iter()
            .all(|word| word.abc_payload().auxiliary == 1),
        "every left operand here is a temporary read for the last time"
    );
    assert_succeeds(source);
}

#[test]
fn record_self_update_moves_the_dead_temporary_and_keeps_aliases() {
    let source = "program SelfUpdate; type P = record A: integer; end; begin mutable var R: P := record A := 1; end; var Copy: P := R; R := R with A := 2; end; if (R.A <> 2) or (Copy.A <> 1) then panic('wrong') end.";
    let program = parse_ok(source);
    let executable = crate::compile(&program).expect("record update compiles");
    assert!(
        executable.executable().code.iter().any(|word| {
            word.opcode() == Ok(fpas_bytecode::Opcode::Move) && word.abc_payload().auxiliary == 1
        }),
        "the copied base of `R with` dies at the update and must be moved"
    );
    assert_succeeds(source);
}
