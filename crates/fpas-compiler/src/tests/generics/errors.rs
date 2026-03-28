use super::*;

// ═══════════════════════════════════════════════════════════════
// NEGATIVE — sema errors
// ═══════════════════════════════════════════════════════════════

#[test]
fn generic_record_wrong_type_arg_count_too_many() {
    let err = compile_err(
        "\
program T;
type Box<T> = record Value: T; end;
begin
  var B: Box of integer, string := record Value := 1; end
end.",
    );
    assert!(
        err.message.contains("type argument"),
        "expected type argument count error, got: {}",
        err.message
    );
}

#[test]
fn generic_record_wrong_type_arg_count_too_few() {
    let err = compile_err(
        "\
program T;
type Pair<A, B> = record First: A; Second: B; end;
begin
  var P: Pair of integer := record First := 1; Second := 2; end
end.",
    );
    assert!(
        err.message.contains("type argument"),
        "expected type argument count error, got: {}",
        err.message
    );
}

#[test]
fn generic_enum_wrong_type_arg_count() {
    let err = compile_err(
        "\
program T;
type Either<L, R> = enum Left(V: L); Right(V: R); end;
begin
  var E: Either of integer := Either.Left(1)
end.",
    );
    assert!(
        err.message.contains("type argument"),
        "expected type argument count error, got: {}",
        err.message
    );
}

#[test]
fn generic_unknown_type_in_type_arg() {
    let err = compile_err(
        "\
program T;
type Box<T> = record Value: T; end;
begin
  var B: Box of Nonexistent := record Value := 1; end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_TYPE);
}

#[test]
fn generic_type_param_not_in_scope_outside() {
    // T should not leak outside the generic type definition
    let err = compile_err(
        "\
program T;
type Box<T> = record Value: T; end;
begin
  var X: T := 42
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_TYPE);
}

#[test]
fn generic_function_type_param_not_in_scope_outside() {
    // T should not leak outside the generic function
    let err = compile_err(
        "\
program T;
function Id<T>(V: T): T;
begin
  return V
end;
begin
  var X: T := 42
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_TYPE);
}

#[test]
fn generic_duplicate_type_param_names_in_record_no_crash() {
    // Even though semantically odd, it should not crash the compiler.
    // The second T simply shadows the first.
    compile_ok(
        "\
program T;
type Box<T, T> = record Value: T; end;
begin
end.",
    );
}

#[test]
fn generic_enum_variant_accepts_any_type_erasure() {
    // Type erasure: the VM does not enforce generic type constraints at
    // runtime, so Maybe.Just('wrong') compiles even when the variable is
    // declared as Maybe of integer. This is expected.
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
type Maybe<T> = enum Just(Value: T); Nothing; end;
begin
  var M: Maybe of integer := Maybe.Just('wrong');
  case M of
    Maybe.Just(V): WriteLn(V);
    Maybe.Nothing: WriteLn('nothing')
  end
end.",
    );
    assert_eq!(out.lines, vec!["wrong"]);
}
