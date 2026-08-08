#![expect(
    clippy::expect_used,
    reason = "program execution round trips use direct fixture assertions"
)]

mod common;

use fpas_program::{decode, encode};

#[test]
fn decoded_register_program_executes_without_sources() {
    let bytes = encode(&common::program_image()).expect("encoded image");
    let image = decode(&bytes).expect("decoded image");
    let mut vm = fpas_vm::Vm::new(image.into_executable());

    assert!(vm.run().is_ok());
}

#[test]
fn real_constant_bits_round_trip_exactly() {
    let image = common::program_image();
    let bytes = encode(&image).expect("encoded image");
    let decoded = decode(&bytes).expect("decoded image");
    let original = &image.executable().executable().constants;
    let round_trip = &decoded.executable().executable().constants;

    assert_eq!(round_trip, original);
}
