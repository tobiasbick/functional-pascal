//! Imported constructors enforce the original record owner, including transparent aliases.

use super::{analyze_unit, parse_unit};
use crate::analyze_program_with_interfaces;
use fpas_diagnostics::codes::{SEMA_PRIVATE_RECORD_MEMBER, SEMA_TYPE_MISMATCH};

#[test]
fn private_defaulted_fields_restrict_direct_and_alias_construction_to_the_owner() {
    let unit = parse_unit(
        "unit Demo.Model;
      public type Secret = record public X: integer; Hidden: integer := 7; end record;
      public type Alias = Secret;
      public function Make(): Secret; begin return Alias(X := 5); end function;
      end unit;",
    );
    let analysis = analyze_unit(&unit, &[]).expect("owner analysis");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.expect("owner interface");
    for target in ["M.Secret", "M.Alias"] {
        let (program, errors) = fpas_parser::parse(&format!(
            "program T; uses Demo.Model as M; begin const P: M.Secret := {target}(X := 1); end."
        ));
        assert!(errors.is_empty(), "{errors:#?}");
        let analysis = analyze_program_with_interfaces(&program, std::slice::from_ref(&interface))
            .expect("consumer analysis");
        assert!(
            analysis
                .errors
                .iter()
                .any(|error| error.code == SEMA_PRIVATE_RECORD_MEMBER),
            "{target}: {:?}",
            analysis.errors
        );
    }
}

#[test]
fn equally_shaped_imported_records_remain_distinct_nominal_types() {
    let interfaces: Vec<_> = ["Demo.First", "Demo.Second"].iter().map(|name| {
        analyze_unit(&parse_unit(&format!("unit {name}; public type Point = record public X: integer; end record; end unit;")), &[]).expect("unit analysis").interface.expect("unit interface")
    }).collect();
    let (program, errors) = fpas_parser::parse(
        "program T; uses Demo.First as A, Demo.Second as B; begin const P: A.Point := B.Point(X := 1); end.",
    );
    assert!(errors.is_empty(), "{errors:#?}");
    let analysis =
        analyze_program_with_interfaces(&program, &interfaces).expect("consumer analysis");
    assert!(
        analysis
            .errors
            .iter()
            .any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{:?}",
        analysis.errors
    );
}
