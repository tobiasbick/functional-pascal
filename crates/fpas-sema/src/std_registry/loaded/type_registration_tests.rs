//! Host resource identity and privacy survive aliases and interface transport.

use fpas_diagnostics::codes::SEMA_PRIVATE_RECORD_MEMBER;
use fpas_parser::{CompilationUnit, Unit};

fn unit(source: &str) -> Unit {
    let (parsed, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("unit fixture");
    };
    unit
}

#[test]
fn file_handle_cannot_be_constructed_or_exposed_as_integer_storage() {
    for declaration in [
        "public function Forge(): Fs.File; begin return Fs.File(); end function;",
        "public function Forge(): Fs.File; begin return Fs.File(_IntrinsicHandle := 7); end function;",
        "public function Read(Value: Fs.File): integer; begin return Value._IntrinsicHandle; end function;",
        "public function Forge(Value: Fs.File): Fs.File; begin return Value with _IntrinsicHandle := 7; end with; end function;",
        "public type Alias = Fs.File; public function Forge(): Alias; begin return Alias(); end function;",
        "public type Wrapper = record public File: Fs.File := Fs.File(); end record;",
        "public type Box of T = record public Item: T; end record; public type FileBox = Box of Fs.File; public function Forge(): FileBox; begin return FileBox(Item := Fs.File()); end function;",
    ] {
        let analysis = crate::analyze_unit(
            &unit(&format!(
                "unit Client; uses Std.Fs as Fs; {declaration} end unit;"
            )),
            &[],
        )
        .expect("semantic analysis");
        assert!(
            analysis
                .metadata
                .errors
                .iter()
                .any(|error| error.code == SEMA_PRIVATE_RECORD_MEMBER),
            "accepted {declaration}: {:?}",
            analysis.metadata.errors
        );
    }
    for declaration in [
        "public function Forge(): Fs.File; begin return 7; end function;",
        "public function Read(Value: Fs.File): integer; begin return Value; end function;",
    ] {
        let analysis = crate::analyze_unit(
            &unit(&format!(
                "unit Client; uses Std.Fs as Fs; {declaration} end unit;"
            )),
            &[],
        )
        .expect("semantic analysis");
        assert!(
            !analysis.metadata.errors.is_empty(),
            "accepted {declaration}"
        );
    }
}

#[test]
fn cached_file_alias_preserves_nominal_owner_and_constructor_privacy() {
    let facade = crate::analyze_unit(
        &unit("unit Resource; uses Std.Fs as Fs; public type File = Fs.File; end unit;"),
        &[],
    )
    .expect("facade analysis");
    assert!(
        facade.metadata.errors.is_empty(),
        "{:?}",
        facade.metadata.errors
    );
    let interface = facade.interface.expect("facade interface");
    let interface = fpas_unit::interface::decode_interface(
        &fpas_unit::interface::encode_interface(&interface).expect("encode interface"),
    )
    .expect("decode interface");
    let copied = crate::analyze_unit(
        &unit("unit Client; uses Resource as Imported; public function Keep(Value: Imported.File): Imported.File; begin return Value; end function; end unit;"),
        std::slice::from_ref(&interface),
    )
    .expect("cached alias analysis");
    assert!(
        copied.metadata.errors.is_empty(),
        "{:?}",
        copied.metadata.errors
    );
    for expression in ["Imported.File()", "Imported.File(_IntrinsicHandle := 7)"] {
        let analysis = crate::analyze_unit(
            &unit(&format!("unit Client; uses Resource as Imported; public function Forge(): Imported.File; begin return {expression}; end function; end unit;")),
            std::slice::from_ref(&interface),
        )
        .expect("cached alias analysis");
        assert!(
            analysis
                .metadata
                .errors
                .iter()
                .any(|error| error.code == SEMA_PRIVATE_RECORD_MEMBER),
            "accepted {expression}: {:?}",
            analysis.metadata.errors
        );
    }
}
