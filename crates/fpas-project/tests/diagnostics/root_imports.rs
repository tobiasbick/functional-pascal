//! Root-import provenance is explicit and independent of the graph's source table.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md`

use super::*;
use fpas_diagnostics::{SourceSpan, codes::PROJECT_UNIT_NOT_EXPORTED};
use fpas_project::{
    UnitGraph, build_unit_graph_for_program_from_parsed_sources, prepare_program_unit_graph,
    resolve_program_units,
};

fn program(source: &str) -> fpas_parser::Program {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    program
}

fn assert_import(
    error: &fpas_project::ProjectError,
    code: DiagnosticCode,
    path: Option<&Path>,
    source: &str,
    name: &str,
    source_id: u32,
) {
    assert_eq!(error.source_path(), path);
    let [diagnostic] = error.diagnostics() else {
        panic!("one import record");
    };
    assert_eq!(diagnostic.code, code);
    assert!(diagnostic.help.is_some());
    let span = diagnostic.span.expect("original import span");
    assert_eq!(span.source_id(), source_id);
    assert_eq!(&source[span.offset()..span.end()], name);
    assert_eq!(span.length(), name.len());
}

fn private_graph(fixture: &Fixture, source: &str) -> (PathBuf, UnitGraph) {
    let main = fixture.write("main.fpas", source);
    fixture.write("api.fpas", "unit Library.Api; end unit;\n");
    fixture.write("hidden.fpas", "unit Library.Hidden; end unit;\n");
    fixture.write("library.fpasprj", "[project]\nname = 'library'\nkind = 'library'\n[sources]\ninclude = ['api.fpas', 'hidden.fpas']\n[exports]\nunits = ['Library.Api']\n");
    let manifest = fixture.write("app.fpasprj", "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['main.fpas']\n[dependencies]\nprojects = ['library.fpasprj']\n");
    let loaded = load_project(&manifest).expect("valid project sources");
    let graph = build_unit_graph_for_program(&main, &loaded.source_files, &loaded.link_meta)
        .expect("program graph");
    (main, graph)
}

#[test]
fn missing_root_import_keeps_the_main_path_and_name_span_in_disk_and_overlay_graphs() {
    let fixture = Fixture::new();
    let source = "program Demo;\nuses Demo.Missing as Missing;\nbegin\nend.\n";
    let main = fixture.write("main.fpas", source);
    let program = program(source);
    let graphs = [
        build_unit_graph_for_program(&main, &[], &ProjectLinkMeta::default()).expect("disk graph"),
        build_unit_graph_for_program_from_parsed_sources(
            &main,
            vec![],
            &ProjectLinkMeta::default(),
        )
        .expect("overlay graph"),
    ];
    for graph in graphs {
        let error =
            resolve_program_units(&graph, &program.uses, Some(&main)).expect_err("missing unit");
        assert_import(
            &error,
            PROJECT_UNKNOWN_UNIT,
            Some(&main),
            source,
            "Demo.Missing",
            0,
        );
        let span = error.diagnostics()[0].span.expect("import span");
        assert_eq!((span.line(), span.column()), (2, 6));
        assert_eq!(
            span,
            SourceSpan::try_from(program.uses[0].unit.span).expect("parser span")
        );
    }
}

#[test]
fn private_root_import_keeps_its_program_path_and_name_span() {
    let fixture = Fixture::new();
    let source = "program Demo;\nuses Library.Hidden as Hidden;\nbegin\nend.\n";
    let (main, graph) = private_graph(&fixture, source);
    let program = program(source);
    let error =
        resolve_program_units(&graph, &program.uses, Some(&main)).expect_err("private unit");
    assert_import(
        &error,
        PROJECT_UNIT_NOT_EXPORTED,
        Some(&main),
        source,
        "Library.Hidden",
        0,
    );
    assert!(
        error.diagnostics()[0]
            .help
            .as_ref()
            .is_some_and(|help| help.contains("[exports].units"))
    );
}

#[test]
fn unavailable_root_path_stays_absent_while_the_original_span_is_preserved() {
    let fixture = Fixture::new();
    let source = "program Demo;\nuses Library.Hidden;\nbegin\nend.\n";
    let (_, graph) = private_graph(&fixture, source);
    for (source, code, name) in [
        (source, PROJECT_UNIT_NOT_EXPORTED, "Library.Hidden"),
        (
            "program Demo;\nuses Library.Missing;\nbegin\nend.\n",
            PROJECT_UNKNOWN_UNIT,
            "Library.Missing",
        ),
    ] {
        let mut program = program(source);
        program.uses[0].unit.span.source_id = 73;
        let error = resolve_program_units(&graph, &program.uses, None).expect_err("invalid import");
        assert_import(&error, code, None, source, name, 73);
    }
}

#[test]
fn multiple_missing_roots_keep_the_selected_imports_own_span() {
    let source = "program Demo;\nuses Std.Console,\n     Demo.First as First,\n     Demo.Second as Second;\nbegin\nend.\n";
    let program = program(source);
    let graph = build_unit_graph(&[], &ProjectLinkMeta::default()).expect("empty graph");
    let main = Path::new("virtual/actual_main.fpas");
    let error =
        resolve_program_units(&graph, &program.uses, Some(main)).expect_err("missing roots");
    assert_import(
        &error,
        PROJECT_UNKNOWN_UNIT,
        Some(main),
        source,
        "Demo.Second",
        0,
    );
    let span = error.diagnostics()[0].span.expect("last root import");
    assert_eq!((span.line(), span.column()), (4, 6));
}

#[test]
fn shared_program_graph_reports_each_entries_own_path() {
    let source = "program Demo;\nuses Demo.Missing;\nbegin\nend.\n";
    let program = program(source);
    let template =
        prepare_program_unit_graph(&[], &ProjectLinkMeta::default(), None).expect("shared graph");
    for main in [
        Path::new("virtual/first_test.fpas"),
        Path::new("virtual/second_test.fpas"),
    ] {
        let graph = template.instantiate(main);
        let error =
            resolve_program_units(&graph, &program.uses, Some(main)).expect_err("missing import");
        assert_import(
            &error,
            PROJECT_UNKNOWN_UNIT,
            Some(main),
            source,
            "Demo.Missing",
            0,
        );
    }
}

#[test]
fn transitive_import_failure_retains_the_unit_path_instead_of_the_program_path() {
    let fixture = Fixture::new();
    let source = "program Demo;\nuses Demo.Dep;\nbegin\nend.\n";
    let main = fixture.write("main.fpas", source);
    let unit_source = "unit Demo.Dep;\nuses Demo.Missing;\nend unit;\n";
    let unit = fixture.write("dep.fpas", unit_source);
    let graph = build_unit_graph_for_program(
        &main,
        std::slice::from_ref(&unit),
        &ProjectLinkMeta::default(),
    )
    .expect("unit graph");
    let program = program(source);
    let error = resolve_program_units(&graph, &program.uses, Some(&main))
        .expect_err("transitive missing import");
    assert_import(
        &error,
        PROJECT_UNKNOWN_UNIT,
        Some(&unit),
        unit_source,
        "Demo.Missing",
        1,
    );
}
