//! Concrete source types refine debug metadata while preserving one erased record layout.

use crate::tests::parse_ok;

#[test]
fn unit_record_method_receivers_and_scalar_parameters_keep_checked_projection_shapes() {
    crate::tests::compiled_unit(
        "unit Reference.Values; const Offset: integer := 1; public type Number = record public Value: integer; public function Next(Self: Number): Number; begin return Number(Value := Self.Value + Offset); end function; end record; public function Make(Value: integer): Number; begin return Number(Value := Value); end function; end unit;",
        &[],
    );
}
use fpas_bytecode::{DebugProjectionKind, DebugType};

#[test]
fn applied_record_projections_do_not_specialize_operational_layouts() {
    let program = parse_ok(
        "program T; type Dollars = distinct integer; type Box of T = record Item: T; end record; var Number: Box of integer := Box(Item := 1); var Text: Box of string := Box(Item := 'one'); var Cash: Box of Dollars := Box(Item := Dollars(2)); begin Number.Item := 3; end.",
    );
    let compiled = crate::compile(&program).expect("concrete projection source");
    let image = compiled.executable();
    assert_eq!(
        image.records.len(),
        1,
        "applications share one runtime layout"
    );
    assert_eq!(
        image.debug_types[image.records[0].fields[0].ty.get() as usize],
        DebugType::Dynamic
    );
    assert_eq!(image.globals[0].ty, image.globals[1].ty);
    assert_eq!(image.globals[1].ty, image.globals[2].ty);
    for (global, expected) in
        image
            .globals
            .iter()
            .zip([DebugType::Integer, DebugType::String, DebugType::Integer])
    {
        let root = global.projection.as_ref().expect("exact global projection");
        assert_eq!(root.ty, global.ty);
        let DebugProjectionKind::Record(fields) = &root.kind else {
            panic!("record projection");
        };
        assert_eq!(image.debug_types[fields[0].ty.get() as usize], expected);
    }
    let DebugProjectionKind::Record(fields) = &image.globals[2]
        .projection
        .as_ref()
        .expect("Cash projection")
        .kind
    else {
        panic!("record projection");
    };
    assert_eq!(
        fields[0].type_name.and_then(|id| image.strings.get(id)),
        Some("Dollars")
    );
}
