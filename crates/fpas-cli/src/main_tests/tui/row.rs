use super::*;

#[test]
fn batched_rows_match_individual_cells_across_clips_and_wide_glyphs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let cwd = create_temp_dir("tui-row-equivalence");
    let library = cwd.join("library");
    let source_glob = root
        .join("lib/Std/**/*.fpas")
        .to_string_lossy()
        .replace('\\', "/");
    write_text(
        &library.join("stdlib.fpasprj"),
        &format!(
            r#"[project]
name = "tui-row-test"
kind = "library"
[exports]
units = ["Std.Tui", "Std.Version", "Std.Tui.Rendering.Canvas"]
[sources]
include = ["{source_glob}"]
"#
        ),
    );
    let standard_library = fpas_project::load_standard_library(&library).expect("test library");
    let graph = fpas_project::prepare_program_unit_graph(
        &[],
        &fpas_project::ProjectLinkMeta::default(),
        Some(&standard_library),
    )
    .expect("test graph");
    let (exit, _, stderr) = support::run_program_with_graph_and_capture_output(
        &root.join("tests/stdlib/tui/cell_grid_row_equivalence.fpas"),
        &graph,
    );
    fs::remove_dir_all(&cwd).expect("remove test directory");
    assert_eq!(exit, 0, "{stderr}");
}
