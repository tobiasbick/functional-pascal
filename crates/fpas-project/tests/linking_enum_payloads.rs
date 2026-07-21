//! Integration tests for linked enum payload types.
//!
//! Documentation: `docs/pascal/program-structure/projects.md`

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect to keep fixture setup compact"
)]

use fpas_project::{build_program, load_project};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

fn temp_dir(name: &str) -> std::path::PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-project-enum-payload-tests-{name}-{}-{id}",
        std::process::id()
    ))
}

fn write(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directories must exist");
    }
    fs::write(path, text).expect("test file must be written");
}

#[test]
fn linked_enum_payload_can_use_an_imported_record_type() {
    let dir = temp_dir("imported-record");
    let project = dir.join("app.fpasprj");

    write(
        &project,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &dir.join("src/ids.fpas"),
        "unit App.Ids;\n\ntype ControlId = record Value: integer; end;\n",
    );
    write(
        &dir.join("src/messages.fpas"),
        "unit App.Messages;\n\nuses App.Ids;\n\ntype Message = enum Action(Source: ControlId); end;\n",
    );
    write(
        &dir.join("src/main.fpas"),
        "program App;\n\nuses App.Ids, App.Messages;\n\nbegin\n  var Msg: Message := Message.Action(record Value := 7; end)\nend.\n",
    );

    let loaded = load_project(&project).expect("project should load");
    let program = build_program(
        loaded.main.as_deref().expect("main"),
        &loaded.source_files,
        &loaded.link_meta,
    )
    .expect("link should succeed");
    let errors = fpas_sema::analyze(&program);
    fs::remove_dir_all(&dir).ok();

    assert!(
        errors.is_empty(),
        "imported enum payload type should resolve: {errors:#?}"
    );
}
