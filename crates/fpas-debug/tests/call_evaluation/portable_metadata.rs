//! Persistence of parameter modes, visible record names, and default routines.
//! See `docs/pascal/tools/debugger.md` and `docs/pascal/program-structure/projects.md`.

use super::*;
use fpas_bytecode::DebugType;
use fpas_program::{Digest, ProgramIdentity, ProgramImage};

#[test]
fn record_defaults_and_parameter_modes_survive_object_linking_and_program_roundtrip() {
    let executable = compile(
        "program PersistDebugTypes;
type Point = record X: integer := 9; end record;
function ReadItem(var Item: integer): integer; begin return Item; end function;
begin end.",
    );
    let object =
        fpas_unit::object::RelocatableObject::from_executable("PersistDebugTypes", executable)
            .expect("object");
    let object = fpas_unit::object::decode_object(
        &fpas_unit::object::encode_object(&object).expect("encode object"),
    )
    .expect("decode object");
    let executable = fpas_linker::link_objects(&[], &object).expect("link object");
    assert!(
        executable
            .executable()
            .debug_types
            .iter()
            .any(|ty| matches!(ty, DebugType::Reference(_)))
    );
    let image = ProgramImage::new(
        ProgramIdentity {
            compiler_version: "test".into(),
            bytecode_version: fpas_bytecode::BYTECODE_VERSION,
            source_hash: Digest::of(b"source"),
            options_hash: Digest::of(b"options"),
            units: vec![],
        },
        vec!["source.fpas".into()],
        vec![Digest::of(b"source")],
        executable,
    )
    .expect("image");
    let decoded = fpas_program::decode(&fpas_program::encode(&image).expect("encode image"))
        .expect("decode image");
    assert_eq!(
        decoded.executable().executable().debug_types,
        image.executable().executable().debug_types
    );
    assert_eq!(
        decoded.executable().executable().records,
        image.executable().executable().records
    );
    let mut session = DebugSession::new(decoded.into_executable()).expect("restored session");
    let expression = DebugExpression::Field {
        base: Box::new(DebugExpression::Record {
            name: "Point".into(),
            fields: vec![],
        }),
        name: "X".into(),
    };
    assert_eq!(
        session
            .evaluate(&expression, None)
            .expect("restored record constructor")
            .value,
        "9"
    );
}
