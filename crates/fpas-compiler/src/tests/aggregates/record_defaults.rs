//! Default-expression identities, contextual types and execution behavior.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`.

use super::super::{assert_succeeds, run_program};

#[test]
fn supplied_fields_precede_defaults_and_preserve_their_written_order() {
    for (fields, message) in [
        ("", "first default"),
        ("First := 42", "second default"),
        ("Second := Fail('supplied second')", "supplied second"),
        (
            "Second := Fail('written first'), First := Fail('written second')",
            "written first",
        ),
    ] {
        let error = run_program(&format!(
            "program Main;
            pure function Fail(Message: string): integer; begin panic(Message); end function;
            type Settings = record
              First: integer := Fail('first default');
              Second: integer := Fail('second default');
            end record;
            begin const Value := Settings({fields}); end program;"
        ))
        .expect_err("the first evaluated failing initializer must abort construction");
        assert!(error.message.contains(message), "{fields}: {error:#?}");
    }
}

#[test]
fn each_construction_calls_only_its_selected_defaults_once() {
    let program = super::super::parse_ok(
        "program Main;
        pure function Answer(): integer; begin return 42; end function;
        type Settings = record First: integer := Answer(); Second: integer := Answer(); end record;
        begin
          const Both := Settings();
          const One := Settings(First := 7);
          const Neither := Settings(Second := 9, First := 8);
        end program;",
    );
    let ir = crate::lower(&program).expect("defaults lower");
    let entry = ir.function(ir.entry).expect("entry function");
    let called_defaults: Vec<_> = entry
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .filter_map(|instruction| match instruction.operation {
            fpas_ir::Operation::CallDirect { function, .. } => {
                Some(ir.function(function).expect("call target").name.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        called_defaults,
        [
            "settings.$default.first",
            "settings.$default.second",
            "settings.$default.second"
        ]
    );
}

#[test]
fn default_names_are_resolved_in_the_declaration_context() {
    assert_succeeds(
        "program Main;
        const Base: integer := 42;
        pure function DefaultValue(): integer; begin return Base; end function;
        type Settings = record
          Count: integer := DefaultValue();
          Callback: function(): integer := function(): integer begin return Base; end function;
        end record;
        function Make(Base: integer): Settings;
        begin return Settings(); end function;
        begin const Value := Make(99);
          if (Value.Count <> 42) or (Value.Callback() <> 42) then panic('default scope'); end if;
        end program;",
    );
}

#[test]
fn callable_record_defaults_keep_closure_identities() {
    assert_succeeds(
        r#"program Defaults;

type Actions = record
  Next: function(Value: integer): integer := function(Value: integer): integer begin
    return Value + 1;
  end function;
  Notify: procedure() := procedure() begin
    null;
  end procedure;
end record;

begin
  const Value: Actions := Actions();
  if Value.Next(41) <> 42 then
    panic('closure default');
  end if;

  Value.Notify();
end program;
"#,
    );
}

#[test]
fn local_record_aliases_keep_default_expression_and_callable_identities() {
    assert_succeeds(
        r#"program Defaults;

type Actions = record
  Count: integer := (1 + 2) * 3;
  Next: function(Value: integer): integer := function(Value: integer): integer begin
    return Value + 1;
  end function;
end record;

type Alias = Actions;

type SecondAlias = aLiAs;

function Make(): SecondAlias;
begin
  return Actions();
end function;

begin
  const Value: SecondAlias := Make();
  const Items: array of (Alias) := [Actions()];
  if (Value.Count <> 9) or (Value.Next(41) <> 42) or (Items[0].Count <> 9) or
     (Items[0].Next(4) <> 5) then
    panic('alias defaults');
  end if;
end program;
"#,
    );
}

#[test]
fn record_defaults_keep_expression_types_in_every_construction_context() {
    assert_succeeds(
        r#"program Defaults;

type Settings = record
  Count: integer := (1 + 2) * 3;
  Scale: real := 1 + 0.5;
  Label: string := 'de' + 'fault';
  Enabled: boolean := not false and (1 < 2);
end record;

function Make(): Settings;
begin
  return Settings();
end function;

procedure Verify(Value: Settings);
begin
  if (Value.Count <> 9) or (Value.Scale <> 1.5) or (Value.Label <> 'default') or not Value.Enabled then
    panic('default expression');
  end if;
end procedure;

begin
  const Local: Settings := Settings();
  const Values: array of (Settings) := [Settings()];
  Verify(Local);
  Verify(Values[0]);
  Verify(Make());
  Verify(Settings());
  const Override: Settings := Settings(Count := 42);
  if Override.Count <> 42 then
    panic('override');
  end if;
end program;
"#,
    );
}

#[test]
fn record_defaults_keep_intrinsic_and_nested_expression_metadata() {
    assert_succeeds(
        r#"program Defaults;

uses Std.Math as Math;
uses Std.Arrays as Arrays;

type Inner = record
  Count: integer := 1 + 2;
end record;

type Settings = record
  Value: integer := Math.Abs(-7) + 1;
  Values: array of (integer) := [];
  InnerValue: Inner := Inner();
end record;

begin
  const Value: Settings := Settings();
  if (Value.Value <> 8) or (Arrays.Length(Value.Values) <> 0) or (Value.InnerValue.Count <> 3) then
    panic('metadata');
  end if;
end program;
"#,
    );
}

#[test]
fn overridden_record_default_is_not_executed() {
    assert_succeeds(
        r#"program Defaults;
pure function Fail(): integer; begin return 1 div 0; end function;

type Settings = record
  Count: integer := Fail();
end record;

begin
  const Value: Settings := Settings(Count := 42);
  if Value.Count <> 42 then
    panic('override');
  end if;
end program;
"#,
    );
}

#[test]
fn record_default_runtime_errors_remain_runtime_errors() {
    let error = run_program(
        r#"program Defaults;
pure function Fail(): integer; begin return 1 div 0; end function;

type Settings = record
  Count: integer := Fail();
end record;

begin
  const Value: Settings := Settings();
end program;
"#,
    )
    .expect_err("executed default must fail at runtime");
    assert!(error.message.contains("zero"), "{error:#?}");
    assert_ne!(error.code.to_string(), "F9001", "{error:#?}");
}
