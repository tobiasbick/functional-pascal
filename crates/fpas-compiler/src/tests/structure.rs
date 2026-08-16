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
