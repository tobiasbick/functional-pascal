//! Last-use transfers preserve live wrapper sources and borrowed intrinsic arguments.
use super::*;
use fpas_runtime::{Intrinsic, Opcode, OptionIntrinsic, ResultIntrinsic};

#[test]
fn wrapper_construction_consumes_temporaries_and_preserves_live_locals() {
    let source = "program Wrappers; begin const Original: option of integer := Some(7); const Nested: option of option of integer := Some(Original); const Temporary: option of option of integer := Some(Some(9)); if Original.Unwrap() <> 7 then panic('original'); end if; if Nested.Unwrap().Unwrap() <> 7 then panic('nested'); end if; if Temporary.Unwrap().Unwrap() <> 9 then panic('temporary'); end if; end.";
    let executable = crate::compile(&parse_ok(source)).expect("wrapper compilation");
    let flags = executable
        .executable()
        .code
        .iter()
        .filter(|word| word.opcode() == Ok(Opcode::MakeSome))
        .map(|word| word.abc_payload().auxiliary)
        .collect::<Vec<_>>();
    assert!(flags.contains(&0), "a live local is borrowed");
    assert!(flags.contains(&1), "a temporary wrapper is transferred");
    assert_succeeds(source);
}

#[test]
fn single_argument_unwrap_intrinsics_borrow_the_parameter_register() {
    let source = "program Borrow; function ReadOption(Value: option of integer): integer; begin return Value.Unwrap(); end function; function ReadResult(Value: result of (integer, string)): integer; begin return Value.Unwrap(); end function; begin if ReadOption(Some(7)) <> 7 then panic('option'); end if; if ReadResult(Ok(9)) <> 9 then panic('result'); end if; end.";
    let executable = crate::compile(&parse_ok(source)).expect("intrinsic compilation");
    let image = executable.executable();
    let mut count = 0;
    for function in &image.functions {
        let code =
            &image.code[function.code.start.get() as usize..function.code.end.get() as usize];
        for word in code {
            if word.opcode() != Ok(Opcode::Intrinsic) {
                continue;
            }
            let operands = word.abc_payload();
            if matches!(
                Intrinsic::from_u16(operands.b),
                Some(
                    Intrinsic::Option(OptionIntrinsic::Unwrap)
                        | Intrinsic::Result(ResultIntrinsic::Unwrap)
                )
            ) {
                assert_eq!(operands.auxiliary, 1);
                let moves = code
                    .iter()
                    .filter(|word| word.opcode() == Ok(Opcode::Move))
                    .map(|word| {
                        let operands = word.abc_payload();
                        (operands.a, operands.b)
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    moves,
                    vec![(operands.c, 0)],
                    "only initialize the parameter binding; no argument-window copy"
                );
                count += 1;
            }
        }
    }
    assert_eq!(count, 2);
    assert_succeeds(source);
}

#[test]
fn pattern_guard_fallbacks_preserve_the_shared_scrutinee() {
    let source = "program Patterns; begin case Some(Some(9)) of when Some(Some(const Value)) if Value = 0: panic('wrong guard'); when Some(Some(const Value)): if Value <> 9 then panic('payload'); end if; when Some(None), None: panic('missing'); end case; end.";
    let executable = crate::compile(&parse_ok(source)).expect("pattern compilation");
    assert!(
        executable
            .executable()
            .code
            .iter()
            .filter(|word| word.opcode() == Ok(Opcode::UnwrapSome))
            .all(|word| word.abc_payload().auxiliary == 0)
    );
    assert_succeeds(source);
}

#[test]
fn borrowed_unwrap_arguments_keep_the_original_error_diagnostics() {
    for (source, message) in [
        (
            "program T; begin const V: option of integer := None; discard V.Unwrap(); end.",
            "Called Option.Unwrap on None",
        ),
        (
            "program T; begin const V: result of (integer, string) := Error('bad'); discard V.Unwrap(); end.",
            "Called Result.Unwrap on Error(bad)",
        ),
    ] {
        let error = run_program(source).expect_err("explicit unwrap failure");
        assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_UNWRAP_FAILURE);
        assert_eq!(error.message, message);
        assert_eq!(error.span.as_ref().expect("source span").line(), 1);
    }
}
