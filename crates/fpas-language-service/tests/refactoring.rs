//! Semantic safety of extraction and import organization.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "source transformation fixtures use direct assertions"
)]

mod support;

use fpas_language_service::{LanguageService, RefactoringKind, SemanticEdit, WorkspaceContext};
use support::TempDirectory;

#[test]
fn extraction_keeps_scope_comments_newlines_and_avoids_existing_names() {
    let temp = TempDirectory::new("extract-preservation");
    let source = "program Extract;\r\nbegin\r\n  const ExtractedValue := '𝄞';\r\n  // Keep this comment.\r\n  const Output := (1 + 2) * 3;\r\nend.\r\n";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let analysis = service.analyze_document(&path).expect("baseline analysis");
    assert!(
        !analysis
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.is_error()),
        "{:?}",
        analysis.diagnostics()
    );
    let start = source.find("(1 + 2)").expect("selection");
    let result = service
        .refactorings(&path, start..start + 7)
        .expect("refactorings");
    let action = result
        .value
        .iter()
        .find(|action| action.kind == RefactoringKind::ExtractVariable);
    assert!(
        action.is_some(),
        "expected extraction for a checked first operand"
    );
    let edited = apply(source, &action.expect("extract action").edits);
    assert_eq!(
        edited,
        source.replace(
            "  const Output := (1 + 2) * 3;",
            "  const ExtractedValue2 := (1 + 2);\r\n  const Output := ExtractedValue2 * 3;"
        )
    );
    assert_eq!(
        service
            .snapshot(&path)
            .expect("unchanged snapshot")
            .source(),
        source
    );
}

#[test]
fn extraction_rejects_changes_to_evaluation_order_frequency_and_reference_arguments() {
    let temp = TempDirectory::new("extract-safety");
    let cases = [
        ("const Value := false and (1 = 2);", "(1 = 2)"),
        ("while 1 < 2 do\n    break;\n  end while;", "1 < 2"),
        ("const Value := 1 + (2 * 3);", "(2 * 3)"),
        ("const Value: Option of integer := None;", "None"),
        ("const Value := if true then 1 else 2 end if;", "1"),
        ("var Count := 0;\n  Change(var Count);", "var Count"),
    ];
    for (statement, selected) in cases {
        let source = format!(
            "program Safety;\nprocedure Change(var Value: integer);\nbegin\n  Value := Value + 1;\nend procedure;\nbegin\n  {statement}\nend.\n"
        );
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let baseline = service
            .analyze_document(&path)
            .expect("valid safety fixture");
        assert!(
            !baseline
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.is_error()),
            "{:?}",
            baseline.diagnostics()
        );
        let start = source.rfind(selected).expect("selection");
        let result = service
            .refactorings(&path, start..start + selected.len())
            .expect("refactorings");
        assert!(
            !result
                .value
                .iter()
                .any(|action| action.kind == RefactoringKind::ExtractVariable),
            "unsafe extraction: {source}"
        );
    }
}

#[test]
fn organize_imports_keeps_aliases_and_effectful_units_but_removes_inert_unused_units() {
    let temp = TempDirectory::new("organize-imports");
    temp.write("app.fpasprj", "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n");
    temp.write(
        "z.fpas",
        "unit Demo.Z; public function Value(): integer; begin return 1; end function; end unit;",
    );
    temp.write(
        "a.fpas",
        "unit Demo.A; public function Value(): integer; begin return 2; end function; end unit;",
    );
    temp.write("unused.fpas", "unit Demo.Unused; end unit;");
    temp.write(
        "state.fpas",
        "unit Demo.State; var State: integer := 0; end unit;",
    );
    let source = "program App;\n\nuses Demo.Z as Last, Demo.A as First, Demo.Unused as Unused, Demo.State as State;\n\nbegin\n  discard Last.Value() + First.Value();\nend.\n";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::load(&temp.path().join("app.fpasprj"));
    let result = service.refactorings(&path, 0..0).expect("organize imports");
    let action = result
        .value
        .iter()
        .find(|action| action.kind == RefactoringKind::OrganizeImports)
        .expect("organize action");
    let edited = apply(source, &action.edits);
    assert_eq!(
        edited,
        source.replace(
            "uses Demo.Z as Last, Demo.A as First, Demo.Unused as Unused, Demo.State as State;",
            "uses Demo.A as First, Demo.Z as Last, Demo.State as State;"
        )
    );
    temp.write("main.fpas", &edited);
    assert!(
        !service
            .refactorings(&path, 0..0)
            .expect("second organization")
            .value
            .iter()
            .any(|action| action.kind == RefactoringKind::OrganizeImports)
    );
}

#[test]
fn organize_imports_does_not_drop_comments_or_transitive_initializers() {
    let temp = TempDirectory::new("organize-effects");
    temp.write("app.fpasprj", "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n");
    temp.write(
        "state.fpas",
        "unit Demo.State; var State: integer := 0; end unit;",
    );
    temp.write(
        "wrapper.fpas",
        "unit Demo.Wrapper; uses Demo.State as State; end unit;",
    );
    temp.write("inert.fpas", "unit Demo.Inert; end unit;");
    for imports in [
        "uses Demo.Wrapper as Wrapper;",
        "uses Demo.Inert as Inert // keep\n;",
    ] {
        let source = format!("program App;\n{imports}\nbegin\nend.\n");
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::load(&temp.path().join("app.fpasprj"));
        assert!(
            service
                .refactorings(&path, 0..0)
                .expect("safe no-op")
                .value
                .is_empty(),
            "{source}"
        );
    }
}

#[test]
fn extraction_stays_inside_loop_bodies_and_before_the_first_static_call_argument() {
    let temp = TempDirectory::new("extract-loop-call");
    let source = "program LoopBody;\nprocedure Consume(Value: integer);\nbegin\n  discard Value;\nend procedure;\nbegin\n  for I: integer := 0 to 2 do\n    Consume(1 + 2);\n  end for;\nend.\n";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let start = source.find("1 + 2").expect("first argument");
    let result = service
        .refactorings(&path, start..start + 5)
        .expect("extraction");
    let action = result
        .value
        .iter()
        .find(|action| action.kind == RefactoringKind::ExtractVariable)
        .expect("loop-local extraction");
    assert_eq!(
        apply(source, &action.edits),
        source.replace(
            "    Consume(1 + 2);",
            "    const ExtractedValue := 1 + 2;\n    Consume(ExtractedValue);"
        )
    );
}

#[test]
fn extraction_does_not_move_an_argument_before_loading_a_mutable_callable() {
    let temp = TempDirectory::new("extract-callable-order");
    let source = "program CallableOrder;\nfunction Read(Value: integer): integer;\nbegin\n  return Value;\nend function;\nvar Reader: function(Value: integer): integer := Read;\nfunction Switch(): integer;\nbegin\n  Reader := Read;\n  return 1;\nend function;\nbegin\n  const Value := Reader(Switch());\nend.\n";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let baseline = service.analyze_document(&path).expect("callable fixture");
    assert!(
        !baseline
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.is_error()),
        "{:?}",
        baseline.diagnostics()
    );
    let start = source.rfind("Switch()").expect("argument");
    assert!(
        service
            .refactorings(&path, start..start + 8)
            .expect("safe no-op")
            .value
            .is_empty()
    );
}

fn apply(source: &str, edits: &[SemanticEdit]) -> String {
    let mut result = source.to_owned();
    let mut edits = edits.iter().collect::<Vec<_>>();
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.span.offset()));
    for edit in edits {
        result.replace_range(edit.span.offset()..edit.span.end(), &edit.new_text);
    }
    result
}
