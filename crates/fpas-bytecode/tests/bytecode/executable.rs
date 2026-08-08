use fpas_bytecode::{Constant, InstructionAddress, SourceId, SourceMap, SourceRun, StringId};

use super::support::{all_opcodes_executable, minimal_executable};

#[test]
fn minimal_candidate_becomes_a_verified_executable() {
    let verified = minimal_executable().verify().expect("fixture must verify");
    assert_eq!(verified.executable().functions.len(), 1);
}

#[test]
fn verifier_accepts_a_fixture_covering_every_active_opcode() {
    let executable = all_opcodes_executable();
    assert_eq!(executable.code.len(), fpas_bytecode::Opcode::ALL.len() + 1);
    assert!(executable.verify().is_ok());
}

#[test]
fn sparse_source_map_uses_closest_preceding_run() {
    let map = SourceMap {
        sources: vec![StringId::new(0)],
        runs: vec![
            SourceRun {
                instruction_start: InstructionAddress::new(2),
                source: SourceId::new(0),
                line: 3,
                column: 4,
            },
            SourceRun {
                instruction_start: InstructionAddress::new(8),
                source: SourceId::new(0),
                line: 9,
                column: 1,
            },
        ],
    };
    assert_eq!(map.lookup(InstructionAddress::new(1)), None);
    assert_eq!(map.lookup(InstructionAddress::new(2)), Some(map.runs[0]));
    assert_eq!(map.lookup(InstructionAddress::new(7)), Some(map.runs[0]));
    assert_eq!(map.lookup(InstructionAddress::new(8)), Some(map.runs[1]));
}

#[test]
fn real_constants_preserve_nan_payload_and_signed_zero_identity() {
    assert_ne!(
        Constant::Real(0.0_f64.to_bits()),
        Constant::Real((-0.0_f64).to_bits())
    );
    assert_ne!(
        Constant::Real(0x7ff8_0000_0000_0001),
        Constant::Real(0x7ff8_0000_0000_0002)
    );
}
