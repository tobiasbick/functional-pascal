//! Compile and run the actual Markdown examples through production CLI paths.
use super::*;

#[test]
fn documented_first_program_builds_and_runs_with_matching_routine_name() {
    let text = include_str!("../../../../../docs/pascal/getting-started/first-program.md");
    let block = text.split("```pascal").nth(1).expect("first program");
    let (_, source) = block.split_once('\n').expect("fenced language");
    let source = source.split_once("```").expect("closing fence").0;
    let cwd = create_temp_dir("documented-first-program");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["main.fpas"]);
    write_text(&cwd.join("main.fpas"), source);
    let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
    fs::remove_dir_all(&cwd).expect("remove fixture");
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "Hello, Pascal!\n");
}

#[test]
fn source_review_documented_project_examples_compile() {
    let text = include_str!("../../../../../docs/pascal/program-structure/projects.md");
    let first = text
        .split("`my-app.fpasprj`:")
        .nth(1)
        .expect("first example");
    let second = text
        .split("`libs/acme-utils/acme-utils.fpasprj`:")
        .nth(1)
        .expect("library example");
    for (label, section, files, entry) in [
        (
            "single",
            first,
            vec!["my-app.fpasprj", "src/main.fpas", "src/math.fpas"],
            "my-app.fpasprj",
        ),
        (
            "library",
            second,
            vec![
                "libs/acme-utils/acme-utils.fpasprj",
                "libs/acme-utils/src/math.fpas",
                "apps/portal/portal.fpasprj",
                "apps/portal/src/main.fpas",
            ],
            "apps/portal/portal.fpasprj",
        ),
    ] {
        let cwd = create_temp_dir(&format!("documented-{label}"));
        let blocks = section.split("```").skip(1).step_by(2);
        for (file, block) in files.iter().zip(blocks) {
            let (_, source) = block.split_once('\n').expect("fenced language");
            write_text(&cwd.join(file), source);
        }
        let (exit, _, stderr) = support::run_cli_args_and_capture_output(
            &[
                "check".into(),
                cwd.join(entry).to_string_lossy().into_owned(),
            ],
            &cwd,
        );
        fs::remove_dir_all(&cwd).expect("remove fixture");
        assert_eq!(exit, 0, "{label}: {stderr}");
    }
}

#[test]
fn source_review_documented_unit_examples_compile() {
    let text = include_str!("../../../../../docs/pascal/program-structure/units.md");
    let cwd = create_temp_dir("documented-units");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/*.fpas"]);
    let sources = text.split("```pascal").skip(1).map(|block| {
        let (_, source) = block.split_once('\n').expect("fenced language");
        source.split_once("```").expect("closing fence").0
    });
    for (file, source) in ["src/utils.fpas", "src/main.fpas"].into_iter().zip(sources) {
        write_text(&cwd.join(file), source);
    }
    let (exit, _, stderr) = support::run_cli_args_and_capture_output(
        &["check".into(), project.to_string_lossy().into_owned()],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("remove fixture");
    assert_eq!(exit, 0, "{stderr}");
}
