//! Allocation limits for small malformed program images.
#![allow(clippy::expect_used, reason = "exact malformed-image fixtures")]
mod common;
use common::{payload_start, program_image, refresh_payload_digest};
use fpas_program::{FormatError, decode, encode};

fn assert_bounded_rejection(bytes: &[u8], field: &'static str) {
    let mut result = None;
    let allocation = allocation_counter::measure(|| {
        result = Some(decode(bytes));
    });
    assert_eq!(
        result.expect("decoder ran").expect_err("malformed image"),
        FormatError::Truncated(field)
    );
    assert!(
        allocation.bytes_total < 64 * 1024,
        "{field} allocated {allocation:?}"
    );
    println!(
        "{field}: allocations={}, bytes={}",
        allocation.count_total, allocation.bytes_total
    );
}

#[test]
fn oversized_section_counts_are_rejected_before_table_reservation() {
    for (index, count, field) in [
        (0_usize, fpas_bytecode::limits::MAX_STRINGS, "string_length"),
        (7, fpas_bytecode::limits::MAX_INSTRUCTIONS, "instruction"),
        (
            8,
            fpas_bytecode::limits::MAX_SOURCE_RUNS,
            "source_instruction",
        ),
    ] {
        let mut bytes = encode(&program_image()).expect("encoded image");
        let entry = payload_start(&bytes) + 4 + index * 16;
        bytes[entry + 12..entry + 16].copy_from_slice(&(count as u32).to_le_bytes());
        refresh_payload_digest(&mut bytes);
        assert_bounded_rejection(&bytes, field);
    }
}

#[test]
fn oversized_header_counts_are_rejected_before_table_reservation() {
    let base = encode(&program_image()).expect("encoded image");
    let compiler_len = u32::from_le_bytes(base[16..20].try_into().expect("compiler length"));
    let unit_count_offset = 20 + compiler_len as usize + 64;
    let mut units = base.clone();
    units[unit_count_offset..unit_count_offset + 4]
        .copy_from_slice(&(fpas_bytecode::limits::MAX_LINKED_UNITS as u32).to_le_bytes());
    assert_bounded_rejection(&units, "unit_name");

    let unit_name_len = u32::from_le_bytes(
        base[unit_count_offset + 4..unit_count_offset + 8]
            .try_into()
            .expect("unit name length"),
    ) as usize;
    let source_count_offset = unit_count_offset + 4 + 4 + unit_name_len + 32;
    let mut sources = base;
    sources[source_count_offset..source_count_offset + 4]
        .copy_from_slice(&(fpas_bytecode::limits::MAX_SOURCE_PATHS as u32).to_le_bytes());
    assert_bounded_rejection(&sources, "source_hash");
}
