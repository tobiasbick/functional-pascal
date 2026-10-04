//! Canonical syntax, comment retention, and structural round trips.

mod common;

#[test]
fn canonical_generics_and_builtin_variants_round_trip() {
    common::assert_round_trip(
        "canonical type applications",
        r#"program Main; function Identity of (T: Equatable)(Value: T): T; begin return Value; end function;
        begin const Value: array of (Result of (Option of (integer), string)) := [Result.Ok(Option.Some(42)), Result.Error('failed')]; end program;"#,
    );
}

#[test]
fn decision_values_round_trip_in_nested_expression_and_callable_positions() {
    for source in [
        r#"program Main; begin const Value: integer := if true then 42 elsif false then 1 else 0 end if; end program;"#,
        "program Main; begin return case Item of // selected value\n when Choice.Present(const Value): if Value > 0 then Value else 0 end if; when Choice.Missing: 0; end case; end program;",
        "program Main; begin (if true then First else Second end if)(); end program;",
    ] {
        common::assert_round_trip("decision expressions", source);
    }
}

#[test]
fn nested_patterns_round_trip_with_qualified_variants_and_explicit_bindings() {
    common::assert_round_trip(
        "recursive patterns",
        "program Main; begin case Item of
        // selected payload
        when Choice.Present(Result.Ok(Option.Some(const Value))) if Value > 0: null;
        when Choice.Present(Result.Ok(Option.Some(_))): null;
        when Choice.Present(Result.Ok(Option.None)): null;
        when Choice.Present(Result.Error(_)): null;
        when Choice.Missing: null; end case; end program;",
    );
}

#[test]
fn named_blocks_imports_and_individual_declarations_round_trip() {
    for source in [
        r#"program P; uses Std.Str as Text; // import
 uses Std.Math as Math; type I = integer; const N: integer := 1; const V: I := N; begin null; end program; // tail"#,
        "unit Example; public type R = record public X: integer; end record; public type E = enum A; B; end enum; public procedure P(); begin null; end procedure; end unit;",
        "program P; begin if true then // then\n null; elsif false then // elsif\n null; else // else\n if false then null; end if; end if; end program;",
        "program P; begin case 1 of when 1: // arm\n null; when 2: null; else null; end case; end program;",
        "program P; begin for I: integer := 0 to 1 do null; end for; for I: integer in [1] do null; end for; while false do null; end while; repeat null; until true; begin null; end; end program;",
        r#"program P;

type R = record
  X: integer;
end record;

begin
  const R1: R := R(X := 1);
  const R2: R := R1 with X := 2; end with;
  const F: function(X: integer): integer := function(X: integer): integer begin
    return X;
  end function;

  F(1);
end program;
"#,
        r#"program P; begin const S: string := 'Span { offset: 123 }'; null; end program;"#,
    ] {
        common::assert_round_trip("named syntax", source);
    }
}

#[test]
fn else_if_and_explicit_blocks_remain_distinct_from_elsif() {
    let source = "program P; begin if true then begin null; end; else if false then null; end if; end if; end program;";
    common::assert_round_trip("branch ownership", source);
    let (unit, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty());
    let formatted = fpas_fmt::format_source(source, &unit).unwrap();
    assert_eq!(formatted.matches("end if;").count(), 2);
    assert!(!formatted.contains("elsif"));
    assert!(formatted.contains("end;"));
}

#[test]
fn nested_named_routines_and_expression_closers_keep_caller_delimiters() {
    for source in [
        "unit Library; public function F(): integer; procedure P(); begin if true then null; else begin null; end; end if; end procedure; begin P(); return 1; end function; end unit;",
        "program P; begin Consume(function(): integer begin // function body\n return 1; end function, procedure() begin // procedure body\n repeat null; until true; end procedure); end program;",
        "program P; type Point = record X: integer; end record; type Empty = record end record; begin Consume(Point(X := 1) with X := 2; end with, Empty()); end program;",
        "program P; type Empty = record end record; begin for I: integer := 2 downto 1 do case I of when 1: if true then null; elsif false then null; else null; end if; else null; end case; end for; end program;",
    ] {
        common::assert_round_trip("nested closers and delimiters", source);
    }
}
