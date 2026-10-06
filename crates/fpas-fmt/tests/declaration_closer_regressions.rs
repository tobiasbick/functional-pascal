//! AP13.3 formatting and comment ownership around named declaration endings.

mod common;

fn format(source: &str) -> String {
    common::assert_round_trip("declaration closers", source);
    let (unit, _) = fpas_parser::parse_compilation_unit(source);
    fpas_fmt::format_source(source, &unit).expect("source and AST agree")
}

#[test]
fn every_named_closer_preserves_comments_before_and_after_it() {
    let source = "unit Demo; // unit header
        type E = enum A;
        // enum close
        end enum; // enum tail
        R = record X: integer;
            function F(self: R): integer; begin return self.X;
            // function close
            end function; // function tail
            procedure P(self: R); begin return;
            // procedure close
            end procedure; // procedure tail
        // record close
        end record; // record tail
        // unit close
        end unit; // unit tail
        // after unit
        ";
    let formatted = format(source);
    for kind in ["enum", "record", "function", "procedure", "unit"] {
        assert_eq!(formatted.matches(&format!("// {kind} close")).count(), 1);
        assert!(
            formatted.contains(&format!("end {kind}; // {kind} tail")),
            "{formatted}"
        );
        let comment = formatted.find(&format!("// {kind} close")).unwrap();
        let closer = formatted.find(&format!("end {kind};")).unwrap();
        assert!(comment < closer, "{formatted}");
    }
    assert!(formatted.starts_with("unit Demo; // unit header\n"));
    assert!(formatted.ends_with("end unit; // unit tail\n// after unit\n"));
}

#[test]
fn empty_unit_keeps_header_and_closer_comments_separate() {
    let formatted = format("unit Demo; // header\n// before end\nend unit; // tail");
    assert_eq!(
        formatted,
        "unit Demo; // header\n\n// before end\nend unit; // tail\n"
    );
}

#[test]
fn declaration_endings_do_not_change_expression_or_scoping_endings() {
    let formatted = format(
        "program T; type R = record X: integer; end record;
        function F(): integer; begin begin return 1; end; end function;
        begin var A: R := record X := 1; end;
        var B: R := A with X := 2; end;
        var C: function(): integer := function(): integer begin return 3; end;
        end.",
    );
    assert_eq!(formatted.matches("end function;").count(), 1);
    assert_eq!(formatted.matches("end record;").count(), 1);
    assert!(!formatted.contains("end with"));
    assert!(formatted.ends_with("end.\n"));
}

#[test]
fn comments_between_closer_keywords_stay_with_their_declaration() {
    let formatted = format(
        "unit Demo; type E = enum A; end // enum ending\n enum;
        R = record end\n// record ending\nrecord;
        function F(): integer; begin return 1; end // function ending\nfunction;
        procedure P(); begin return; end\n// procedure ending\nprocedure;
        end // unit ending\nunit; // unit tail",
    );
    for kind in ["enum", "record", "function", "procedure", "unit"] {
        let comment = formatted.find(&format!("// {kind} ending")).unwrap();
        let closer = formatted.find(&format!("end {kind};")).unwrap();
        assert!(comment < closer, "{formatted}");
    }
}
