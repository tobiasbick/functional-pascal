//! Forward and recursive types execute with their complete layouts and ordered values.

use super::assert_succeeds;

#[test]
fn forward_types_variants_methods_and_defaults_execute() {
    assert_succeeds(include_str!(
        "../../../../../tests/runner/type_order_test.fpas"
    ));
}

#[test]
fn container_aliases_in_nominal_cycles_preserve_typed_layouts() {
    for definitions in [
        "type Children = array of Node; type Node = record Value: integer; Children: Children; end record;",
        "type Node = record Value: integer; Children: Children; end record; type Children = array of Node;",
    ] {
        assert_succeeds(&format!(
            "program T; {definitions}
          begin const Leaf: Node := Node( Value := 7, Children := [] );
          const Root: Node := Node( Value := 9, Children := [Leaf] );
          if Root.Children[0].Value <> 7 then panic('recursive layout'); end if; end."
        ));
    }
}
