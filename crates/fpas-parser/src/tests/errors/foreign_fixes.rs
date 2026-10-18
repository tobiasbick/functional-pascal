//! Fixes attached to parser habit diagnostics: applying every machine-applicable fix, pass
//! after pass, turns the foreign source into FPAS that parses without errors.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Fixes)

use crate::ParseDiagnostic;
use crate::tests::parse_with_errors;
use fpas_diagnostics::{Applicability, Diagnostic, apply_machine_applicable};

/// Passes after which a source must parse cleanly.
const MAX_PASSES: usize = 10;

fn diagnostics(source: &str) -> Vec<Diagnostic> {
    let (_, diagnostics) = parse_with_errors(source);
    diagnostics
        .iter()
        .map(ParseDiagnostic::as_diagnostic)
        .cloned()
        .collect()
}

/// Applies machine-applicable fixes until none is left and returns the fixed text.
fn fixed(source: &str) -> String {
    let mut text = source.to_owned();
    for _ in 0..MAX_PASSES {
        let diagnostics = diagnostics(&text);
        let (next, applied) =
            apply_machine_applicable(&text, diagnostics.iter().flat_map(|d| &d.fixes));
        if applied == 0 {
            break;
        }
        text = next;
    }
    text
}

/// Asserts that `source` is fixed into `expected`, which parses without errors.
fn assert_fixed(source: &str, expected: &str) {
    let text = fixed(source);
    assert_eq!(text, expected, "{source}");
    let remaining = diagnostics(&text);
    assert!(remaining.is_empty(), "{text}: {remaining:#?}");
}

fn program(body: &str) -> String {
    format!(
        "program P;\n\nprocedure Q(X: integer);\nbegin\n  var Y: integer := 0;\n{body}\nend procedure;\n\nbegin\nend."
    )
}

/// The single fix of the single diagnostic of `source`.
fn only_fix(source: &str) -> fpas_diagnostics::Fix {
    let diagnostics = diagnostics(source);
    assert_eq!(diagnostics.len(), 1, "{source}: {diagnostics:#?}");
    let [fix] = diagnostics[0].fixes.as_slice() else {
        panic!("{source}: {:#?}", diagnostics[0].fixes);
    };
    fix.clone()
}

#[test]
fn pascal_if_statements_become_fpas_conditionals() {
    assert_fixed(
        &program("  if X > 0 then\n    Y := 1\n  else\n    Y := 2;"),
        &program("  if X > 0 then\n    Y := 1;\n  else\n    Y := 2;\n  end if;"),
    );
    assert_fixed(
        &program("  if X > 0 then begin\n    Y := 1;\n  end;"),
        &program("  if X > 0 then\n    Y := 1;\n  end if;"),
    );
    assert_fixed(
        &program(
            "  if X > 0 then\n  begin\n    Y := 1;\n  end\n  else\n  begin\n    Y := 2;\n  end;",
        ),
        &program("  if X > 0 then\n    Y := 1;\n  else\n    Y := 2;\n  end if;"),
    );
}

#[test]
fn statements_after_a_pascal_body_stay_outside() {
    assert_fixed(
        &program("  if X > 0 then\n    Y := 1\n  else\n    Y := 2;\n  Y := 3;"),
        &program("  if X > 0 then\n    Y := 1;\n  else\n    Y := 2;\n  end if;\n  Y := 3;"),
    );
}

#[test]
fn pascal_loops_become_fpas_loops() {
    assert_fixed(
        &program("  while Y < X do\n  begin\n    Y := Y + 1;\n  end;"),
        &program("  while Y < X do\n    Y := Y + 1;\n  end while;"),
    );
    assert_fixed(
        &program("  for I := 1 to X do\n  begin\n    Y := Y + I;\n  end;"),
        &program("  for I: integer := 1 to X do\n    Y := Y + I;\n  end for;"),
    );
}

#[test]
fn pascal_case_arms_get_when_and_the_named_ending() {
    assert_fixed(
        &program("  case X of\n    1: Y := 1;\n    2, 3: Y := 2;\n  else\n    Y := 0;\n  end;"),
        &program(
            "  case X of\n    when 1: Y := 1;\n    when 2, 3: Y := 2;\n  else\n    Y := 0;\n  end case;",
        ),
    );
}

#[test]
fn untyped_for_in_and_inner_errors_have_no_fix() {
    let source = program("  for I in Items do\n  begin\n    Y := I;\n  end;");
    assert!(diagnostics(&source).iter().all(|d| d.fixes.is_empty()));
}

#[test]
fn routine_headers_and_results_are_fixed() {
    assert_fixed(
        "program P;\n\nfunction F: integer;\nbegin\n  Result := 42;\nend function;\n\nbegin\nend.",
        "program P;\n\nfunction F(): integer;\nbegin\n  return 42;\nend function;\n\nbegin\nend.",
    );
    assert_fixed(
        "program P;\n\nprocedure Q(const S: string);\nbegin\nend procedure;\n\nbegin\nend.",
        "program P;\n\nprocedure Q(S: string);\nbegin\nend procedure;\n\nbegin\nend.",
    );
}

#[test]
fn ada_routines_and_programs_get_fpas_headers_and_endings() {
    assert_fixed(
        "program P;\n\nfunction F return integer is\nbegin\n  return 1;\nend F;\n\nbegin\nend.",
        "program P;\n\nfunction F(): integer;\nbegin\n  return 1;\nend function;\n\nbegin\nend.",
    );
    assert_fixed("program P is\nbegin\nend P;", "program P;\nbegin\nend.");
}

#[test]
fn declaration_groups_get_one_keyword_each() {
    assert_fixed(
        "program P;\n\nconst A: integer := 1;\n  B: integer := 2;\n\nbegin\nend.",
        "program P;\n\nconst A: integer := 1; const B: integer := 2;\n\nbegin\nend.",
    );
    assert_fixed(
        "program P;\n\ntype Color = (Red, Green);\n\nbegin\nend.",
        "program P;\n\ntype Color = enum Red; Green; end enum;\n\nbegin\nend.",
    );
}

#[test]
fn guessed_initializers_need_review() {
    let fix = only_fix(
        "program P;\n\nfunction F(): integer;\nvar\n  N: integer;\nbegin\n  return N;\nend function;\n\nbegin\nend.",
    );
    assert_eq!(fix.applicability, Applicability::NeedsReview);
    assert_eq!(
        fix.apply("program P;\n\nfunction F(): integer;\nvar\n  N: integer;\nbegin\n  return N;\nend function;\n\nbegin\nend.")
            .as_deref(),
        Some("program P;\n\nfunction F(): integer;\n\nbegin var N: integer := 0;\n  return N;\nend function;\n\nbegin\nend.")
    );
}

#[test]
fn angle_type_arguments_become_of() {
    assert_fixed(
        "program P;\n\ntype Pair = Result<integer, string>;\n\nbegin\nend.",
        "program P;\n\ntype Pair = Result of (integer, string);\n\nbegin\nend.",
    );
}
