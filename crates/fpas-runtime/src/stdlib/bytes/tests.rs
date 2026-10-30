//! Public range failures use ordinary VM diagnostics rather than typed buffer errors.
use crate::execution::Vm;

#[test]
fn invalid_buffer_sizes_indices_and_ranges_fail_with_array_bounds_diagnostics() {
    for body in [
        "discard Bytes.Create(-1);",
        "discard B[3];",
        "discard B.Slice(-1, 0);",
        "discard B.Slice(3, 1);",
        "discard B.Slice(9223372036854775807, 1);",
        "discard B.Fill(1, 0, -1);",
        "discard B.CopyFrom(B, 2, 0, 2);",
        "discard B.CopyFrom(B, 0, 2, 2);",
        "discard B.SetAtInPlace(3, 0);",
        "discard B.ReserveInPlace(-1);",
        "discard B.ResizeInPlace(-1);",
    ] {
        let source = format!(
            "program Bounds; uses Std.Bytes as Bytes; begin var B := Bytes.Create(3).Unwrap(); {body} end."
        );
        let (program, errors) = fpas_parser::parse(&source);
        assert!(errors.is_empty(), "{errors:?}");
        let image = fpas_compiler::compile(&program).expect("range fixture compiles");
        let error = Vm::new(image).run().expect_err("invalid byte range");
        assert_eq!(
            error.code,
            fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
            "{body}: {error:?}"
        );
    }
}

#[test]
fn direct_byte_index_failures_match_array_index_failures() {
    for index in [-1, 3, i64::MAX] {
        let mut codes = Vec::new();
        for initializer in ["Bytes.Create(3).Unwrap()", "[0, 0, 0]"] {
            let source = format!(
                "program Index; uses Std.Bytes as Bytes; begin const B := {initializer}; discard B[{index}]; end."
            );
            let (program, errors) = fpas_parser::parse(&source);
            assert!(errors.is_empty(), "{errors:?}");
            let image = fpas_compiler::compile(&program).expect("index fixture compiles");
            codes.push(Vm::new(image).run().expect_err("invalid index").code);
        }
        assert_eq!(codes[0], codes[1], "index={index}");
    }
}
