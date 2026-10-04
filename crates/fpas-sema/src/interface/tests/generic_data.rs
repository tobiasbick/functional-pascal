//! Generic data descriptors survive independent unit analysis and alias reexports.

use super::*;
use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_PRIVATE_TYPE_IN_PUBLIC_SIGNATURE};

#[test]
fn generic_template_and_concrete_alias_metadata_round_trip() {
    let original = analyze_unit(&parse_unit("unit Demo.Model;
        public type Box of (T: Equatable) = record public Value: T; public Children: array of (Box of (T)); end record;
        public type Lookup of (T) = enum Found(Value: T); Missing; end enum;
        public function ReadBox of (U: Equatable)(Item: Box of (U)): U;
        begin return Item.Value; end function;
        end unit;"), &[]).expect("model analysis");
    assert!(
        original.metadata.errors.is_empty(),
        "{:#?}",
        original.metadata.errors
    );
    let original = original.interface.expect("model interface");
    let serialized = fpas_unit::interface::encode_interface(&original).expect("encode");
    let restored = fpas_unit::interface::decode_interface(&serialized).expect("decode");
    assert_eq!(restored, original);
    let facade = analyze_unit(
        &parse_unit(
            "unit Demo.Facade; uses Demo.Model as Model;
        public type IntegerBox = Model.Box of (integer);
        public function ReadValue(Item: IntegerBox): integer;
        begin return Model.ReadBox(Item); end function;
        end unit;",
        ),
        &[restored.clone()],
    )
    .expect("facade analysis");
    assert!(
        facade.metadata.errors.is_empty(),
        "{:#?}",
        facade.metadata.errors
    );
    let facade = facade.interface.expect("facade interface");
    let consumer = crate::analyze_unit_with_interface_support(
        &parse_unit(
            "unit Demo.Consumer;
        uses Demo.Facade as Facade;
        public function ReadValue(Item: Facade.IntegerBox): integer;
        begin return Item.Children[0].Value; end function;
        end unit;",
        ),
        &[facade.clone()],
        &[restored, facade],
    )
    .expect("consumer analysis");
    assert!(
        consumer.metadata.errors.is_empty(),
        "{:#?}",
        consumer.metadata.errors
    );
}

#[test]
fn imported_generic_type_constraints_remain_enforced() {
    let model = analyze_unit(
        &parse_unit(
            "unit Demo.Model;
        public type Box of (T: Numeric) = record public Value: T; end record;
        end unit;",
        ),
        &[],
    )
    .expect("model")
    .interface
    .expect("interface");
    let consumer = analyze_unit(
        &parse_unit(
            "unit Demo.Consumer; uses Demo.Model as Model;
        public type Invalid = Model.Box of (string);
        end unit;",
        ),
        &[model],
    )
    .expect("consumer");
    assert!(
        consumer
            .metadata
            .errors
            .iter()
            .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
        "{:#?}",
        consumer.metadata.errors
    );
}

#[test]
fn public_generic_parameters_shadow_private_short_types_only() {
    let allowed = analyze_unit(
        &parse_unit(
            "unit Demo.Model;
        type T = string;
        public type Box of (T) = record public Value: T; end record;
        public function Identity of (T)(Value: T): T; begin return Value; end function;
        end unit;",
        ),
        &[],
    )
    .expect("analysis");
    assert!(
        allowed.metadata.errors.is_empty(),
        "{:#?}",
        allowed.metadata.errors
    );
    let forbidden = analyze_unit(
        &parse_unit(
            "unit Demo.Model;
        type Secret = string;
        public type Box of (T) = record public Value: T; end record;
        public type Hidden = Box of (Secret);
        end unit;",
        ),
        &[],
    )
    .expect("diagnostics");
    assert!(
        forbidden
            .metadata
            .errors
            .iter()
            .any(|error| error.code == SEMA_PRIVATE_TYPE_IN_PUBLIC_SIGNATURE),
        "{:#?}",
        forbidden.metadata.errors
    );
}
