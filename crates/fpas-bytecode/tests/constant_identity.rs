#![expect(
    clippy::expect_used,
    reason = "constant fixtures use expect for compact setup"
)]

use fpas_bytecode::{Chunk, PersistentValue, Value};

#[test]
fn constant_pool_distinguishes_signed_zero_bits() {
    let mut chunk = Chunk::new();

    assert_eq!(Value::Real(0.0), Value::Real(-0.0));

    let positive = chunk.add_constant(Value::Real(0.0)).expect("positive zero");
    let negative = chunk
        .add_constant(Value::Real(-0.0))
        .expect("negative zero");

    assert_eq!((positive, negative), (0, 1));
    assert_eq!(real_bits(&chunk, positive), Some(0.0_f64.to_bits()));
    assert_eq!(real_bits(&chunk, negative), Some((-0.0_f64).to_bits()));
}

#[test]
fn persistent_real_json_round_trip_preserves_signed_zero_bits() {
    for bits in [0.0_f64.to_bits(), (-0.0_f64).to_bits()] {
        let persistent = PersistentValue::Real(bits);
        let encoded = serde_json::to_vec(&persistent).expect("serialize real constant");
        let decoded: PersistentValue =
            serde_json::from_slice(&encoded).expect("deserialize real constant");

        assert_eq!(decoded, persistent);
        let decoded_bits = match decoded.to_value() {
            Value::Real(value) => Some(value.to_bits()),
            _ => None,
        };
        assert_eq!(decoded_bits, Some(bits));
    }
}

fn real_bits(chunk: &Chunk, index: u16) -> Option<u64> {
    match chunk.constants().get(index as usize) {
        Some(Value::Real(value)) => Some(value.to_bits()),
        _ => None,
    }
}
