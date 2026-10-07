//! Canonical keywords, visibility, and comments for individual declarations.

#![allow(
    clippy::expect_used,
    reason = "test fixtures fail fast with direct assertions for diagnostic clarity"
)]

mod common;

#[test]
fn inline_constants_preserve_comments_closures_and_complete_prefixes() {
    let source = "program Values;
        begin // declaration order
            const First: integer := ReadValue(); // computed
            const Callback: function(): integer := function(): integer
                begin const Saved: integer := First; return Saved; end function;
            if true then const Second: integer := Callback(); end if;
        end.";
    common::assert_round_trip("inline constants", source);
    let (unit, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:?}");
    let formatted = fpas_fmt::format_source(source, &unit).expect("format");
    assert!(
        formatted.contains("const First: integer := ReadValue(); // computed"),
        "{formatted}"
    );
    assert!(formatted.contains("const Saved: integer := First;"));
}

#[test]
fn every_declaration_retains_its_complete_prefix_and_source_order() {
    let source = "unit Values;
        // first type
        public type A = integer; public type B = string;
        public const C: integer := 1; public const D: integer := 2;
        const Hidden: integer := 3;
        public const E: integer := C; public const F: integer := D;
        public var G: integer := 0; public var H: integer := 0;
        const Immutable: integer := 1;
        end unit;";
    common::assert_round_trip("individual declarations", source);
    common::assert_golden(
        "individual declarations",
        source,
        "unit Values;\n\n\
         // first type\n\
         public type A = integer;\n\n\
         public type B = string;\n\n\
         public const C: integer := 1;\n\
         public const D: integer := 2;\n\n\
         const Hidden: integer := 3;\n\n\
         public const E: integer := C;\n\
         public const F: integer := D;\n\n\
         public var G: integer := 0;\n\
         public var H: integer := 0;\n\n\
         const Immutable: integer := 1;\n\
         end unit;\n",
    );
}
