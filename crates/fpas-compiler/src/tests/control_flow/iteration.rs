//! Independent dictionary copies retain ordinary lookup failure behavior.
//! See `docs/pascal/language/control-flow/for-in.md`.

use super::super::run_program;
use fpas_diagnostics::codes::RUNTIME_DICT_KEY_NOT_FOUND;

#[test]
fn for_in_independent_dictionary_copy_retains_missing_key_failures() {
    let error = run_program(
        "program RemovedIterationKey;
        begin
          var Values: dict of string to integer := ['keep': 1, 'removed': 2, 'last': 3];
          const Source := Values;
          for Key: string in Source do
            Values.RemoveInPlace('removed');
            discard Values[Key];
          end for;
        end.",
    )
    .expect_err("a key from an independent source can be absent from the changed dictionary");
    assert_eq!(error.code, RUNTIME_DICT_KEY_NOT_FOUND, "{error:?}");
}
