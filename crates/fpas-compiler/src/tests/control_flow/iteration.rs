//! Snapshotted dictionary keys retain ordinary current-value lookup failures.
//! See `docs/pascal/language/control-flow/for-in.md`.

use super::super::run_program;
use fpas_diagnostics::codes::RUNTIME_DICT_KEY_NOT_FOUND;

#[test]
fn for_in_removed_snapshot_key_fails_when_read_from_the_current_dictionary() {
    let error = run_program(
        "program RemovedIterationKey;
        begin
          var Values: dict of string to integer := ['keep': 1, 'removed': 2, 'last': 3];
          for Key: string in Values do
            Values.RemoveInPlace('removed');
            discard Values[Key];
          end for;
        end.",
    )
    .expect_err("a retained iteration key can be absent from the current dictionary");
    assert_eq!(error.code, RUNTIME_DICT_KEY_NOT_FOUND, "{error:?}");
}
