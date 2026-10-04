//! Default-expression identities, contextual types and execution behavior.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`.

use super::super::{assert_succeeds, run_program};

#[test]
fn callable_record_defaults_keep_closure_identities() {
    assert_succeeds(
        "program Defaults;\n\ntype Actions = record\n  Next: function(Value: integer): integer := function(Value: integer): integer begin\n    return Value + 1;\n  end function;\n  Notify: procedure() := procedure() begin\n    null;\n  end procedure;\nend record;\n\nbegin\n  var Value: Actions := Actions();\n  if Value.Next(41) <> 42 then\n    panic('closure default');\n  end if;\n\n  Value.Notify();\nend program;\n",
    );
}

#[test]
fn local_record_aliases_keep_default_expression_and_callable_identities() {
    assert_succeeds(
        "program Defaults;\n\ntype Actions = record\n  Count: integer := (1 + 2) * 3;\n  Next: function(Value: integer): integer := function(Value: integer): integer begin\n    return Value + 1;\n  end function;\nend record;\n\ntype Alias = Actions;\n\ntype SecondAlias = aLiAs;\n\nfunction Make(): SecondAlias;\nbegin\n  return Actions();\nend function;\n\nbegin\n  var Value: SecondAlias := Make();\n  var Items: array of (Alias) := [Actions()];\n  if (Value.Count <> 9) or (Value.Next(41) <> 42) or (Items[0].Count <> 9) or\n     (Items[0].Next(4) <> 5) then\n    panic('alias defaults');\n  end if;\nend program;\n",
    );
}

#[test]
fn record_defaults_keep_expression_types_in_every_construction_context() {
    assert_succeeds(
        "program Defaults;\n\ntype Settings = record\n  Count: integer := (1 + 2) * 3;\n  Scale: real := 1 + 0.5;\n  Label: string := 'de' + 'fault';\n  Enabled: boolean := not false and (1 < 2);\nend record;\n\nfunction Make(): Settings;\nbegin\n  return Settings();\nend function;\n\nprocedure Verify(Value: Settings);\nbegin\n  if (Value.Count <> 9) or (Value.Scale <> 1.5) or (Value.Label <> 'default') or not Value.Enabled then\n    panic('default expression');\n  end if;\nend procedure;\n\nbegin\n  var Local: Settings := Settings();\n  var Values: array of (Settings) := [Settings()];\n  Verify(Local);\n  Verify(Values[0]);\n  Verify(Make());\n  Verify(Settings());\n  var Override: Settings := Settings(Count := 42);\n  if Override.Count <> 42 then\n    panic('override');\n  end if;\nend program;\n",
    );
}

#[test]
fn record_defaults_keep_intrinsic_and_nested_expression_metadata() {
    assert_succeeds(
        "program Defaults;\n\nuses Std.Math as Math;\nuses Std.Arrays as Arrays;\n\ntype Inner = record\n  Count: integer := 1 + 2;\nend record;\n\ntype Settings = record\n  Value: integer := Math.Abs(-7) + 1;\n  Values: array of (integer) := [];\n  InnerValue: Inner := Inner();\nend record;\n\nbegin\n  var Value: Settings := Settings();\n  if (Value.Value <> 8) or (Arrays.Length(Value.Values) <> 0) or (Value.InnerValue.Count <> 3) then\n    panic('metadata');\n  end if;\nend program;\n",
    );
}

#[test]
fn overridden_record_default_is_not_executed() {
    assert_succeeds(
        "program Defaults;\n\ntype Settings = record\n  Count: integer := 1 div 0;\nend record;\n\nbegin\n  var Value: Settings := Settings(Count := 42);\n  if Value.Count <> 42 then\n    panic('override');\n  end if;\nend program;\n",
    );
}

#[test]
fn record_default_runtime_errors_remain_runtime_errors() {
    let error = run_program(
        "program Defaults;\n\ntype Settings = record\n  Count: integer := 1 div 0;\nend record;\n\nbegin\n  var Value: Settings := Settings();\nend program;\n",
    )
    .expect_err("executed default must fail at runtime");
    assert!(error.message.contains("zero"), "{error:#?}");
    assert_ne!(error.code.to_string(), "F9001", "{error:#?}");
}
