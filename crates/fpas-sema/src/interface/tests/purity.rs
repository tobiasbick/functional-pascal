//! Pure capabilities of imported data include transitive intrinsic nominal types.

use super::*;

#[test]
fn imported_data_resolves_intrinsic_types_without_exposing_their_names() {
    let interface = analyze_unit(
        &parse_unit(
            "unit Demo.Model; uses Std.Console as Console;
        public type Key = record public Kind: Console.KeyKind; end record; end unit;",
        ),
        &[],
    )
    .unwrap()
    .interface
    .unwrap();
    let analysis = analyze_unit(&parse_unit("unit Demo.Consumer; uses Demo.Model as Model;
        public pure function Count(Value: Model.Key): integer; begin return 1; end function; end unit;"), &[interface.clone()]).unwrap();
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let analysis = analyze_unit(
        &parse_unit(
            "unit Demo.Consumer; uses Demo.Model as Model;
        public type Unexpected = Std.Console.KeyKind; end unit;",
        ),
        &[interface],
    )
    .unwrap();
    assert!(!analysis.metadata.errors.is_empty());
}

#[test]
fn imported_intrinsic_resources_remain_forbidden_in_pure_signatures() {
    let interface = analyze_unit(
        &parse_unit(
            "unit Demo.Model; uses Std.Net as Net;
        public type Handle = record public Connection: Net.Connection; end record; end unit;",
        ),
        &[],
    )
    .unwrap()
    .interface
    .unwrap();
    let analysis = analyze_unit(&parse_unit("unit Demo.Consumer; uses Demo.Model as Model;
        public pure function Count(Value: Model.Handle): integer; begin return 1; end function; end unit;"), &[interface]).unwrap();
    assert!(
        analysis
            .metadata
            .errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH)
    );
}
