use super::{analyze_unit, parse_unit};
use fpas_unit::interface::{decode_interface, encode_interface};

#[test]
fn public_clause_identity_order_and_interface_hash_survive_roundtrip() {
    let source = "unit Api;
        public procedure Guard(X: integer); requires X > 0; requires X < 10; ensures X > 0; ensures X < 10; begin null; end procedure;
        procedure Hidden(); requires false; begin null; end procedure;
        public type Box = record public Value: integer;
            public function Check(Self: Box): integer; ensures (Checked): Checked = Self.Value; begin return Self.Value; end function;
            static procedure Hidden(); requires false; begin null; end procedure;
        end record;
        public type Original = record public Value: integer;
            public procedure Check(Self: Original); requires Self.Value > 0; begin null; end procedure;
        end record;
        public type Alias = Original; end unit;";
    let analysis = analyze_unit(&parse_unit(source), &[]).expect("analysis");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.expect("interface");
    assert_eq!(
        interface
            .routine_contracts
            .iter()
            .map(|contract| contract.qualified_name.as_str())
            .collect::<Vec<_>>(),
        ["Api.Box.Check", "Api.Guard", "Api.Original.Check"]
    );
    assert_eq!(
        interface.routine_contracts[1].requires,
        ["requires X > 0;", "requires X < 10;"]
    );
    assert_eq!(
        interface.routine_contracts[0].ensures,
        ["ensures (Checked): Checked = Self.Value;"]
    );
    assert_eq!(
        interface.routine_contracts[1].ensures,
        ["ensures X > 0;", "ensures X < 10;"]
    );
    let bytes = encode_interface(&interface).expect("encode");
    assert_eq!(decode_interface(&bytes).expect("decode"), interface);
    let changed = analyze_unit(&parse_unit(&source.replace("X > 0", "X > 1")), &[])
        .expect("analysis")
        .interface
        .expect("interface");
    assert_ne!(
        interface.digest().expect("hash"),
        changed.digest().expect("hash")
    );
    let changed = analyze_unit(
        &parse_unit(&source.replace("ensures X > 0", "ensures X > 1")),
        &[],
    )
    .expect("analysis")
    .interface
    .expect("interface");
    assert_ne!(
        interface.digest().expect("hash"),
        changed.digest().expect("hash")
    );
}
