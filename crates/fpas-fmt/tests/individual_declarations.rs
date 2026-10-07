//! Canonical keywords, visibility, and comments for individual declarations.

mod common;

#[test]
fn every_declaration_retains_its_complete_prefix_and_source_order() {
    let source = "unit Values;
        // first type
        public type A = integer; public type B = string;
        public const C: integer := 1; public const D: integer := 2;
        const Hidden: integer := 3;
        public var E: integer := C; public var F: integer := D;
        public mutable var G: integer := 0; public mutable var H: integer := 0;
        var Immutable: integer := 1;
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
         public var E: integer := C;\n\
         public var F: integer := D;\n\n\
         public mutable var G: integer := 0;\n\
         public mutable var H: integer := 0;\n\n\
         var Immutable: integer := 1;\n\
         end unit;\n",
    );
}
