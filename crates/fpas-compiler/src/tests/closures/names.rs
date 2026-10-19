//! Anonymous executable names follow lexical ownership, not semantic check order.

use crate::tests::parse_ok;

fn names(image: &fpas_bytecode::Executable) -> Vec<String> {
    image
        .functions
        .iter()
        .filter_map(|function| image.strings.get(function.name))
        .filter(|name| name.contains("$closure_"))
        .map(str::to_ascii_lowercase)
        .collect()
}

#[test]
fn anonymous_names_follow_source_order_and_complete_nested_owners() {
    let source = r#"
program Names;
function Outer(Seed: integer): function(): integer;
  function Unused(): integer; begin return 0; end function;
begin
  if true then
    discard [1].Map(function(N: integer): integer begin return N; end function);
  end if;
  const Later := function(): integer
    function Helper(): integer;
    begin
      const Leaf := function(): integer begin return Seed; end function;
      return Leaf();
    end function;
  begin
    const Nested := function(): integer begin return Seed; end function;
    return Helper() + Nested();
  end function;
  return Later;
end function;
function Other(): function(): integer;
begin return function(): integer begin return 3; end function; end function;
begin
  const A := Outer(7);
  const B := Other();
  if (A() <> 14) or (B() <> 3) then panic('wrong nested result'); end if;
end.
"#;
    let program = parse_ok(source);
    let compiled = crate::compile(&program).expect("nested names compile");
    let expected = vec![
        "outer.$closure_1",
        "outer.$closure_2",
        "outer.$closure_2.helper",
        "outer.$closure_2.helper.$closure_1",
        "outer.$closure_2.$closure_1",
        "other.$closure_1",
    ];
    assert_eq!(names(compiled.executable()), expected);
    fpas_vm::Vm::new(compiled)
        .run()
        .expect("nested closures run");
    // A second parse creates different AST addresses while keeping source identities.
    assert_eq!(
        names(
            crate::compile(&parse_ok(source))
                .expect("fresh compile")
                .executable()
        ),
        expected
    );
}

#[test]
fn retained_default_aliases_count_one_literal_and_names_survive_object_linking() {
    let program = parse_ok(
        r#"
program Defaults;
type Holder = record
  Read: function(): integer := function(): integer begin return 7; end function;
end record;
type Alias = Holder;
const Global: function(): integer := function(): integer begin return 9; end function;
begin
  const A := Holder();
  const B := Alias();
  const Local := function(): integer begin return 11; end function;
  if (A.Read() <> 7) or (B.Read() <> 7) or (Global() <> 9) or (Local() <> 11) then
    panic('wrong default result');
  end if;
end.
"#,
    );
    let compiled = crate::compile(&program).expect("default names compile");
    let expected = vec![
        "$entry_defaults.$closure_1",
        "$entry_defaults.$closure_2",
        "$entry_defaults.$closure_3",
    ];
    assert_eq!(names(compiled.executable()), expected);
    fpas_vm::Vm::new(compiled).run().expect("defaults run");
    let object = crate::compile_object(&program).expect("object compiles");
    let encoded = fpas_unit::object::encode_object(&object).expect("encode object");
    let decoded = fpas_unit::object::decode_object(&encoded).expect("decode object");
    let linked = fpas_linker::link_objects(&[], &decoded).expect("link object");
    assert_eq!(names(linked.executable()), expected);
    fpas_vm::Vm::new(linked).run().expect("linked defaults run");
}

#[test]
fn generic_routines_and_generic_helpers_inside_closures_keep_distinct_owners() {
    let source = r#"
program GenericNames;
function Make<T>(Value: T; Seed: integer): function(): integer;
begin
  const Other := function(): integer
    function Identity<U>(Item: U): U; begin return Item; end function;
  begin return Identity(Seed); end function;
  if Other() <> Seed then panic('wrong sibling helper'); end if;
  return function(): integer
    function Identity<U>(Item: U): U; begin return Item; end function;
  begin
    discard Value;
    return Identity(Seed);
  end function;
end function;
begin
  const A := Make(1, 7);
  const B := Make('text', 9);
  if (A() <> 7) or (B() <> 9) then panic('wrong generic result'); end if;
end.
"#;
    let compiled = crate::compile(&parse_ok(source)).expect("generic closures compile");
    assert_eq!(
        names(compiled.executable()),
        vec![
            "make.$closure_1",
            "make.$closure_1.identity",
            "make.$closure_2",
            "make.$closure_2.identity"
        ]
    );
    fpas_vm::Vm::new(compiled)
        .run()
        .expect("generic closures run");
}
