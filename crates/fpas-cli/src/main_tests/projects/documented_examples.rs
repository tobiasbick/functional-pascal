//! Compile the actual Markdown project examples, including visibility modifiers.
use super::*;

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
