//! Canonical syntax, comment retention, and structural round trips.

mod common;

#[test]
fn named_blocks_imports_and_individual_declarations_round_trip() {
    for source in [
        "program P; uses Std.Str as Text; // import\n uses Std.Math as Math; type I = integer; const N: integer := 1; var V: I := N; begin null; end program; // tail",
        "unit Example; public type R = record public X: integer; end record; public type E = enum A; B; end enum; public procedure P(); begin null; end procedure; end unit;",
        "program P; begin if true then // then\n null; elsif false then // elsif\n null; else // else\n if false then null; end if; end if; end program;",
        "program P; begin case 1 of when 1: // arm\n null; when 2: null; else null; end case; end program;",
        "program P; begin for I: integer := 0 to 1 do null; end for; for I: integer in [1] do null; end for; while false do null; end while; repeat null; until true; begin null; end; end program;",
        "program P; type R = record X: integer; end record; begin var R1: R := record X := 1; end record; var R2: R := R1 with X := 2; end with; var F: function(X: integer): integer := function(X: integer): integer begin return X; end function; F(1); end program;",
        "program P; begin var S: string := 'Span { offset: 123 }'; null; end program;",
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
        "program P; begin Consume(record X := 1; end record with X := 2; end with, record end record); end program;",
        "program P; type Empty = record end record; begin for I: integer := 2 downto 1 do case I of when 1: if true then null; elsif false then null; else null; end if; else null; end case; end for; end program;",
    ] {
        common::assert_round_trip("nested closers and delimiters", source);
    }
}
