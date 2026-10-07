//! Recursive forward types preserve encoded interfaces and member privacy.

use super::{analyze_unit, parse_unit};
use fpas_diagnostics::codes::{SEMA_PRIVATE_RECORD_MEMBER, SEMA_PRIVATE_TYPE_IN_PUBLIC_SIGNATURE};

#[test]
fn forward_recursive_types_survive_encoded_interfaces() {
    let dependency = parse_unit(
        "unit Demo.Tree;
      public function Create(): Node; begin return Node.Empty(); end function;
      public type Node = record public Children: array of Edge;
        public static function Empty(): Node; begin return record Children := []; end; end function;
        public function Count(Self: Node): integer; begin return 1; end function;
      end record;
      public type Edge = enum Stop; More(Next: Option of Node); end enum;
      public type Alias = Node;
      end unit;",
    );
    let analysis = analyze_unit(&dependency, &[]).unwrap();
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.unwrap();
    let alias = interface
        .symbols
        .iter()
        .find(|symbol| symbol.name == "Alias")
        .unwrap();
    let fpas_unit::interface::InterfaceType::Record(record) = &alias.ty else {
        panic!("alias must retain its record descriptor")
    };
    assert!(record.methods.iter().any(|method| method.name == "Count"));
    assert!(
        record
            .static_routines
            .iter()
            .any(|method| method.name == "Empty")
    );
    let bytes = fpas_unit::interface::encode_interface(&interface).unwrap();
    let interface = fpas_unit::interface::decode_interface(&bytes).unwrap();
    let consumer = parse_unit(
        "unit Demo.Consumer; uses Demo.Tree;
      public function Run(): integer; begin
        var Root: Alias := Create();
        var Next: Edge := Edge.More(Some(Root));
        discard Next; return Root.Count() + Alias.Empty().Count();
      end function; end unit;",
    );
    let analysis = analyze_unit(&consumer, &[interface]).unwrap();
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
}

#[test]
fn early_references_do_not_expose_private_types_or_members() {
    let source = parse_unit(
        "unit Demo.Hidden;
      public function Create(): Hidden; begin return record Value := 1; end; end function;
      type Hidden = record Value: integer; end record; end unit;",
    );
    let analysis = analyze_unit(&source, &[]).unwrap();
    assert!(
        analysis
            .metadata
            .errors
            .iter()
            .any(|error| error.code == SEMA_PRIVATE_TYPE_IN_PUBLIC_SIGNATURE)
    );
    let dependency = parse_unit(
        "unit Demo.Secret;
      public type Box = record public Value: integer;
        static function Hidden(): integer; begin return 7; end function;
      end record; end unit;",
    );
    let interface = analyze_unit(&dependency, &[]).unwrap().interface.unwrap();
    let consumer = parse_unit(
        "unit Demo.Consumer; uses Demo.Secret;
      function Run(): integer; begin return Wrapper.Hidden(); end function;
      type Wrapper = Box; end unit;",
    );
    let analysis = analyze_unit(&consumer, &[interface]).unwrap();
    assert!(
        analysis
            .metadata
            .errors
            .iter()
            .any(|error| error.code == SEMA_PRIVATE_RECORD_MEMBER),
        "{:?}",
        analysis.metadata.errors
    );
}
