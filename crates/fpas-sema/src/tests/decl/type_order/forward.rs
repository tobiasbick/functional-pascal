use super::check_ok;

#[test]
fn later_types_variants_methods_and_defaults_are_available() {
    check_ok(
        "program T;
      const Initial: State := State.Ready;
      function Create(): Team; begin return Team.Empty(); end function;
      type Team = record Members: array of Member; Status: State := Initial;
        static function Empty(): Team; begin return record Members := []; end; end function;
      end record;
      type Member = record Home: Option of Team; end record;
      type State = enum Ready; Busy; end enum;
      begin var Group: Team := Create(); discard Group; end.",
    );
}

#[test]
fn aliases_and_enum_variants_are_visible_before_their_declarations() {
    check_ok(
        "program T; const Value: Alias := Alias.Ready;
      type Alias = Later; type Later = State; type State = enum Ready; Busy; end enum;
      begin discard Value; end.",
    );
}

#[test]
fn nominal_recursion_through_aliases_is_finite() {
    check_ok(
        "program T; type Link = Option of Node;
      type Node = record Next: Link; end record;
      begin var Root: Node := record Next := None; end;
      var Next: Node := record Next := Some(Root); end; discard Next; end.",
    );
}

#[test]
fn mutual_records_enums_and_aliases_work_in_every_declaration_order() {
    let definitions = [
        "type Node = record Value: Choice; end record;",
        "type Choice = enum Empty; More(Next: Children); end enum;",
        "type Children = array of Node;",
    ];
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        check_ok(&format!(
            "program T; {} {} {} begin var Root: Node := record Value := Choice.Empty; end; discard Root; end.",
            definitions[order[0]], definitions[order[1]], definitions[order[2]]
        ));
    }
}

#[test]
fn method_type_parameters_do_not_change_other_type_definitions() {
    check_ok(
        "program T;
      type First = record
        static function Identity<T: Numeric>(Value: T): T; begin return Value; end function;
        static function Make(): Later; begin return record Value := State.Ready; end; end function;
      end record;
      type Later = record Value: State; end record;
      type State = enum Ready; end enum;
      begin discard First.Identity(1); discard First.Make(); end.",
    );
}

#[test]
fn generic_routines_can_infer_recursive_forward_types() {
    check_ok(
        "program T;
      function Identity<T>(Value: T): T; begin return Value; end function;
      function Create(): Node; begin return record Children := []; end; end function;
      type Node = record Children: array of Node; end record;
      begin var Value: Node := Identity(Create()); discard Value; end.",
    );
}

#[test]
fn recursive_enum_aliases_expose_variants_in_both_orders() {
    for definitions in [
        "type Alias = Chain; type Chain = enum Empty; Link(Next: Alias); end enum;",
        "type Chain = enum Empty; Link(Next: Alias); end enum; type Alias = Chain;",
    ] {
        check_ok(&format!(
            "program T; {definitions} begin var Value: Alias := Alias.Link(Alias.Empty); discard Value; end."
        ));
    }
}

#[test]
fn recursive_fields_keep_complete_method_signatures() {
    check_ok(
        "program T;
      function Inspect(Value: A): integer; begin return Value.Next.Value(); end function;
      type A = record Next: B; end record;
      type B = record Back: Option of A;
        function Value(Self: B): integer; begin return 7; end function;
      end record;
      begin var Value: A := record Next := record Back := None; end; end;
      var Bound: function(): integer := Value.Next.Value;
      discard Inspect(Value); discard Bound(); end.",
    );
}
