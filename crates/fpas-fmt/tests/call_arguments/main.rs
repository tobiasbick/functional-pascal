//! Formatting and comment preservation for named reference arguments.

#[path = "../common/mod.rs"]
mod common;

#[test]
fn typed_record_calls_round_trip_without_reordering_fields_or_comments() {
    let source = "program Records; type Point = record X: integer; Y: integer := 2; end record; type Position = Point; begin const P: Point := Position(Y:=4, // written first\n X:=1); const Points: array of Point := [Point(X:=3)]; end.";
    common::assert_round_trip("typed record construction", source);
    let (unit, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:#?}");
    let formatted = fpas_fmt::format_source(source, &unit).expect("format constructor");
    assert!(
        formatted.find("Y := 4").expect("first field")
            < formatted.find("X := 1").expect("second field")
    );
    assert!(formatted.contains("// written first"));
    assert!(formatted.contains("Point(X := 3)"));
}

#[test]
fn named_var_arguments_keep_markers_paths_comments_and_written_order() {
    let source = "program Named;
      begin
        Store(Last:=3,Target:=var Items[Index()],First:=1); // order
        Swap(B:=var P.Y,A:=var Counter);
        Make().Store(Value:= // reference
          var Counter);
      end.";
    common::assert_round_trip("named var arguments", source);
    let (unit, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:#?}");
    let formatted = fpas_fmt::format_source(source, &unit).expect("format source");
    for expected in [
        "Store(Last := 3, Target := var Items[Index()], First := 1); // order",
        "Swap(B := var P.Y, A := var Counter);",
        "var Counter",
    ] {
        assert!(formatted.contains(expected), "{formatted}");
    }
}
