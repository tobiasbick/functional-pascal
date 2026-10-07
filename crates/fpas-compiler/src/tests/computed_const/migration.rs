//! Behavior preserved when immutable variable consumers become constants.

use super::*;

#[test]
fn migration_preserves_guard_bindings_and_nested_shadowing() {
    for initializer in ["1", "ReadValue()"] {
        let (keyword, binding) = ("const", "Matched");
        let source = format!(
            "program T;
                type Item = record N: integer; end record;
                function ReadValue(): integer; begin return 1; end function;
                begin
                  {keyword} N: integer := {initializer};
                  const Object: Item := record N := 7; end;
                  var Score: integer := 0;
                  case 2 of
                    when {binding} if {binding} > 0:
                      begin
                        const ReadMatched: function(): integer :=
                          function(): integer
                          begin return {binding} + Object.N; end function;
                        Score := ReadMatched();
                        begin
                          const N: integer := 40;
                          Score := Score + N;
                        end;
                      end;
                    else Score := -1;
                  end case;
                  if Score <> 49 or N <> 1 then panic('migration changed scope'); end if;
                end."
        );
        assert_succeeds(&source);
    }
}

#[test]
fn migrated_guard_regression_runs_in_the_fpas_suite() {
    assert_succeeds(include_str!(
        "../../../../../tests/runner/immutable_binding_migration_test.fpas"
    ));
}
