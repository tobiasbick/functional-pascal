//! Distinct identities and constant conversions survive compiled-unit interfaces.
//!
//! Documentation: `docs/pascal/language/types/distinct-types.md`

use super::*;
use crate::analyze_program_with_interfaces;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_unit::interface::{ConstantValue, decode_interface, encode_interface};

fn interface(source: &str) -> fpas_unit::interface::UnitInterface {
    let analysis = analyze_unit(&parse_unit(source), &[]).expect("unit analysis");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.expect("interface");
    decode_interface(&encode_interface(&interface).expect("encode")).expect("decode")
}

fn consumer_errors(
    source: &str,
    interfaces: &[fpas_unit::interface::UnitInterface],
) -> Vec<crate::SemaError> {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:#?}");
    analyze_program_with_interfaces(&program, interfaces)
        .expect("consumer analysis")
        .errors
}

#[test]
fn exported_distinct_types_keep_identity_owner_underlying_type_and_constants() {
    let ids = interface(
        "unit Demo.Ids;
        public type UserId = distinct integer;
        public const Admin: UserId := UserId(7);
        end unit;",
    );
    let symbol = |name: &str| {
        ids.symbols
            .iter()
            .find(|symbol| symbol.qualified_name.eq_ignore_ascii_case(name))
            .unwrap_or_else(|| panic!("missing {name}: {:#?}", ids.symbols))
    };
    let InterfaceType::Distinct(user_id) = &symbol("Demo.Ids.UserId").ty else {
        panic!("{:#?}", symbol("Demo.Ids.UserId"));
    };
    assert_eq!(user_id.name, "UserId");
    assert_eq!(user_id.owner_unit.as_deref(), Some("Demo.Ids"));
    assert_eq!(*user_id.underlying, InterfaceType::Integer);
    let admin = symbol("Demo.Ids.Admin");
    assert_eq!(
        admin.kind,
        SymbolKind::Constant(Some(ConstantValue::Integer(7)))
    );
    assert!(matches!(admin.ty, InterfaceType::Distinct(_)));

    let errors = consumer_errors(
        "program T; uses Demo.Ids as Ids;
        begin
          const Raw: integer := integer(Ids.Admin);
          case 7 of when Raw: null; else null; end case;
          const Plain: integer := Ids.Admin;
        end.",
        std::slice::from_ref(&ids),
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(errors[0].message.contains("found `UserId`"), "{errors:#?}");
}

#[test]
fn equally_named_distinct_types_from_different_units_are_not_interchangeable() {
    let interfaces: Vec<_> = ["Demo.First", "Demo.Second"]
        .iter()
        .map(|name| {
            interface(&format!(
                "unit {name}; public type Id = distinct integer; end unit;"
            ))
        })
        .collect();
    let errors = consumer_errors(
        "program T; uses Demo.First as A, Demo.Second as B;
        begin const Value: A.Id := B.Id(1); end.",
        &interfaces,
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{errors:#?}");
}
