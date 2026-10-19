//! Nominal scalar display metadata survives erasure, object conversion, and linking.

use fpas_bytecode::DebugType;

use crate::tests::parse_ok;

#[test]
fn declared_scalar_names_survive_compilation_object_round_trip_and_linking() {
    let program = parse_ok(
        r#"
program DebugScalarNames;
type UserId = distinct integer;
type UserAlias = UserId;
type Percent = 0..100;
type Shade = enum Red; Blue; end enum;
type Counter = integer;
type State = record
  Id: UserId;
  Progress: Percent;
  Color: Shade;
  Count: Counter;
end record;
var Id: UserAlias := UserId(3);
var Progress: Percent := Percent(50);
var Color: Shade := Shade.Red;
var Count: Counter := 2;
var Current: State := State(Id := Id, Progress := Progress, Color := Color, Count := Count);
begin
  Id := UserId(4);
  Progress := Percent(51);
  Color := Shade.Blue;
end.
"#,
    );
    let compiled = crate::compile(&program).expect("nominal scalar fixture compiles");
    assert_names(compiled.executable());
    let object = crate::compile_object(&program).expect("nominal scalar object compiles");
    let encoded = fpas_unit::object::encode_object(&object).expect("object encoding");
    let decoded = fpas_unit::object::decode_object(&encoded).expect("object decoding");
    assert_eq!(decoded, object);
    let linked = fpas_linker::link_objects(&[], &decoded).expect("object links");
    assert_names(linked.executable());
}

fn assert_names(image: &fpas_bytecode::Executable) {
    for (name, expected) in [
        ("Id", Some("UserId")),
        ("Progress", Some("Percent")),
        ("Color", Some("Shade")),
        ("Count", None),
        ("Current", None),
    ] {
        let global = image
            .globals
            .iter()
            .find(|global| {
                image.strings.get(global.name).is_some_and(|candidate| {
                    candidate
                        .rsplit('.')
                        .next()
                        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(name))
                })
            })
            .expect("global metadata");
        assert_eq!(
            global.type_name.and_then(|id| image.strings.get(id)),
            expected
        );
        if expected.is_some() {
            assert_eq!(
                image.debug_types[global.ty.get() as usize],
                DebugType::Integer
            );
        }
    }
    let record = image.records.first().expect("State layout");
    for (field, expected) in
        record
            .fields
            .iter()
            .zip([Some("UserId"), Some("Percent"), Some("Shade"), None])
    {
        assert_eq!(
            field.type_name.and_then(|id| image.strings.get(id)),
            expected
        );
        assert_eq!(
            image.debug_types[field.ty.get() as usize],
            DebugType::Integer
        );
    }
}
