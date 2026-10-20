//! Sibling projects retain one movable source tree for image debugging.

use super::*;

const MAIN: &str = "program Main;\nuses Shared as S;\nbegin\n  S.Advance();\n  if S.Total <> 7 then panic('wrong library value'); end if;\nend.\n";
const LIBRARY: &str = "unit Shared;\npublic var Total: integer := 1;\npublic procedure Advance();\nbegin\n  Total := 7;\nend procedure;\nend unit;\n";

fn sibling_projects() -> Fixture {
    let f = Fixture::new(MAIN);
    f.write("app/src/main.fpas", MAIN);
    f.write("shared/src/main.fpas", LIBRARY);
    f.write(
        "app/app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='src/main.fpas'\n[sources]\ninclude=['src/*.fpas']\n[dependencies]\nprojects=['../shared/shared.fpasprj']\n",
    );
    f.write(
        "shared/shared.fpasprj",
        "[project]\nname='shared'\nkind='library'\n[sources]\ninclude=['src/*.fpas']\n",
    );
    f
}

fn build(f: &Fixture) -> String {
    let output = f
        .command()
        .args(["build", "app/app.fpasprj"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(
        text.contains(&format!(
            "Debug source root: {}",
            f.0.canonicalize().unwrap().display()
        )),
        "{text}"
    );
    text
}

fn trace_image(f: &Fixture, target: &str, root: &str, exit: i32) -> Value {
    f.run(
        &[
            "debug",
            target,
            "--source-root",
            root,
            "--schedule-seed",
            "42",
            "--trace",
            "Shared.Total",
        ],
        exit,
    )
}

#[test]
fn absolute_breakpoint_paths_bind_to_the_loaded_source_identity() {
    let f = sibling_projects();
    build(&f);
    let library = f.0.join("shared/src/main.fpas");
    let paths = [
        "shared/src/main.fpas".to_string(),
        library.to_string_lossy().into_owned(),
        library.to_string_lossy().replace('\\', "/"),
        library
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
    ];
    for target in ["app/app.fpasprj", "app/app.fpascp"] {
        for path in &paths {
            let location = format!("{path}:5");
            let stopped = f.run(
                &["debug", target, "--source-root", ".", "--break", &location],
                0,
            );
            assert_eq!(stopped["outcome"], "breakpoint", "{target}: {path}");
            assert_eq!(
                stopped["stop"]["location"]["source"],
                "shared/src/main.fpas"
            );
            assert_eq!(stopped["stop"]["location"]["line"], 5);
            assert_eq!(stopped["globals"][0]["value"]["scalar"], "1");
        }
    }

    // A file with the same name and contents outside the target is not an alias.
    let other = Fixture::new(LIBRARY);
    let location = format!("{}:5", other.0.join("main.fpas").display());
    let rejected = f.run(&["debug", "app/app.fpasprj", "--break", &location], 1);
    assert_eq!(rejected["diagnostics"][0]["code"], "breakpoint_unbound");
}

#[test]
fn sibling_sources_share_paths_cache_reuse_legacy_rebuild_and_relocated_image_traces() {
    let f = sibling_projects();
    assert!(build(&f).contains("Built program"));
    let artifact = f.0.join("app/app.fpascp");
    let image = fpas_program::decode(&fs::read(&artifact).unwrap()).unwrap();
    assert_eq!(
        image.source_paths(),
        &["app/src/main.fpas", "shared/src/main.fpas"]
    );
    let image_stamp = fs::metadata(&artifact).unwrap().modified().unwrap();
    let object = f.0.join("shared/src/main.fpascu");
    let object_stamp = fs::metadata(&object).unwrap().modified().unwrap();
    assert!(build(&f).contains("Reused program"));
    assert_eq!(
        image_stamp,
        fs::metadata(&artifact).unwrap().modified().unwrap()
    );

    // An older producer used paths rooted in the program project, including .. .
    let legacy = fpas_program::ProgramImage::new(
        image.identity().clone(),
        vec!["src/main.fpas".into(), "../shared/src/main.fpas".into()],
        image.source_hashes().to_vec(),
        image.executable().clone(),
    )
    .unwrap();
    fs::write(&artifact, fpas_program::encode(&legacy).unwrap()).unwrap();
    assert!(build(&f).contains("Built program"));
    assert_eq!(
        object_stamp,
        fs::metadata(&object).unwrap().modified().unwrap()
    );

    let project = f.run(
        &[
            "debug",
            "app/app.fpasprj",
            "--schedule-seed",
            "42",
            "--trace",
            "Shared.Total",
        ],
        0,
    );
    let image = trace_image(&f, "app/app.fpascp", ".", 0);
    for key in ["trace", "schedule", "sources", "tasks"] {
        assert_eq!(project[key], image[key], "{key}");
    }
    assert_eq!(image["trace"]["complete"], true);
    let values: Vec<_> = image["trace"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["new_value"]["scalar"].clone())
        .collect();
    assert_eq!(
        values,
        vec![Value::String("1".into()), Value::String("7".into())]
    );
    let stopped = f.run(
        &[
            "debug",
            "app/app.fpascp",
            "--source-root",
            ".",
            "--break",
            "shared/src/main.fpas:5",
        ],
        0,
    );
    assert_eq!(stopped["outcome"], "breakpoint");
    assert_eq!(
        stopped["stop"]["location"]["source"],
        "shared/src/main.fpas"
    );
    assert_eq!(stopped["globals"][0]["value"]["scalar"], "1");

    fs::create_dir(f.0.join("relocated")).unwrap();
    for directory in ["app", "shared"] {
        fs::rename(f.0.join(directory), f.0.join("relocated").join(directory)).unwrap();
    }
    let relocated = trace_image(&f, "relocated/app/app.fpascp", "relocated", 0);
    for key in ["trace", "schedule", "sources", "tasks"] {
        assert_eq!(image[key], relocated[key], "relocated {key}");
    }
}

#[test]
fn image_rejects_a_changed_sibling_source_before_execution() {
    let f = sibling_projects();
    build(&f);
    f.write(
        "shared/src/main.fpas",
        &format!("{LIBRARY}// changed after build\n"),
    );
    let report = trace_image(&f, "app/app.fpascp", ".", 1);
    assert_eq!(report["diagnostics"][0]["code"], "target_setup");
    assert!(
        report["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("stale")
    );
    assert_eq!(report["schedule"]["status"], "not_started");
    assert!(report["tasks"].as_array().unwrap().is_empty());
}

#[cfg(unix)]
#[test]
fn image_rejects_a_sibling_symlink_escape_even_with_identical_source_bytes() {
    let f = sibling_projects();
    build(&f);
    let outside = Fixture::new(LIBRARY);
    let library = f.0.join("shared/src/main.fpas");
    fs::remove_file(&library).unwrap();
    std::os::unix::fs::symlink(outside.0.join("main.fpas"), &library).unwrap();
    let report = trace_image(&f, "app/app.fpascp", ".", 1);
    assert!(
        report["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("escapes source root")
    );
    assert_eq!(report["schedule"]["status"], "not_started");
    assert!(report["tasks"].as_array().unwrap().is_empty());
}
