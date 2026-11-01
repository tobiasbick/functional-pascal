//! Public preview artifacts, isolated preparation and surviving source publication.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "real process fixtures assert exact lifecycle outcomes"
)]

use serde_json::{Value, json};
#[path = "refactor/parameter.rs"]
mod parameter;
#[path = "refactor/safety.rs"]
mod safety;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

const UNIT: &str = "unit Demo.Math;\npublic function Answer(): integer;\nbegin\n  return 42;\nend function;\nend unit;\n";
const MAIN: &str = "program App;\r\nuses Demo.Math as M;\r\nbegin\r\n  discard '😀 Answer'; discard M.answer();\r\n  discard M.ANSWER(); // Answer\r\nend.\r\n";

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let f = Self(std::env::temp_dir().join(format!(
            "fpas-refactor-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        f.write(
            "suite.fpasworkspace",
            "[workspace]\nname='suite'\nmembers=['lib/lib.fpasprj','app/app.fpasprj']\n",
        );
        f.write(
            "lib/lib.fpasprj",
            "[project]\nname='lib'\nkind='library'\n[sources]\ninclude=['*.fpas']\n",
        );
        f.write("app/app.fpasprj", "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[dependencies]\nprojects=['../lib/lib.fpasprj']\n[sources]\ninclude=['*.fpas']\n");
        f.write(
            "std/stdlib.fpasprj",
            "[project]\nname='stdlib'\nkind='library'\n[sources]\ninclude=['*.fpas']\n",
        );
        f.write("std/custom.fpas", "unit Std.Custom; end unit;\n");
        f.write("lib/math.fpas", UNIT);
        f.write("app/main.fpas", MAIN);
        f
    }
    fn write(&self, name: &str, source: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    fn output(&self, args: &[&str], cwd: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_fpas"))
            .arg("refactor")
            .args(args)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .output()
            .unwrap()
    }
    fn run(&self, args: &[&str], exit: i32) -> (Value, Vec<u8>) {
        decode(self.output(args, &self.0), exit)
    }
    fn preview(&self, extra: &[&str], exit: i32) -> (Value, Vec<u8>) {
        let mut args = vec![
            "rename",
            "--symbol",
            "Demo.Math.Answer",
            "--name",
            "Compute",
            "--context",
            "suite.fpasworkspace",
            "--std-lib",
            "std",
        ];
        args.extend_from_slice(extra);
        self.run(&args, exit)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn decode(output: Output, exit: i32) -> (Value, Vec<u8>) {
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.status.code(),
        Some(exit),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    (
        serde_json::from_slice(&output.stdout).expect("one JSON envelope"),
        output.stdout,
    )
}

#[test]
fn rename_preview_artifact_apply_and_repeat_preserve_exact_bytes_across_working_directories() {
    let f = Fixture::new();
    let (preview, bytes) = f.preview(&["--out", "rename.plan.json"], 0);
    assert_eq!(preview["schema_version"], 2);
    assert_eq!(preview["result"]["status"], "prepared");
    assert_eq!(fs::read(f.0.join("rename.plan.json")).unwrap(), bytes);
    assert_eq!(f.preview(&["--out", "rename.plan.json"], 0).1, bytes);
    assert_eq!(fs::read_to_string(f.0.join("app/main.fpas")).unwrap(), MAIN);
    assert_eq!(
        preview["result"]["plan"]["edits"].as_array().unwrap().len(),
        3
    );
    let artifact = f.0.join("rename.plan.json").to_string_lossy().into_owned();
    let (applied, _) = decode(
        f.output(&["apply", "--plan", &artifact], &f.0.join("app")),
        0,
    );
    assert_eq!(applied["result"]["status"], "applied");
    assert!(
        applied["result"]["publication"]["paths"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["state"] == "replaced")
    );
    assert_eq!(
        fs::read_to_string(f.0.join("lib/math.fpas")).unwrap(),
        UNIT.replace("Answer", "Compute")
    );
    assert_eq!(
        fs::read_to_string(f.0.join("app/main.fpas")).unwrap(),
        MAIN.replace("M.answer()", "M.Compute()")
            .replace("M.ANSWER()", "M.Compute()")
    );
    let (repeat, _) = f.run(&["apply", "--plan", "rename.plan.json"], 0);
    assert_eq!(repeat["result"]["status"], "already_applied");
    assert!(repeat["result"]["publication"].is_null());
    f.write("app/extra.fpas", "unit Demo.Extra; end unit;");
    let (stale, _) = f.run(&["apply", "--plan", "rename.plan.json"], 1);
    assert!(
        stale["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "stale_plan")
    );
}

#[test]
fn artifact_collisions_membership_and_tampered_plans_never_publish_source() {
    let f = Fixture::new();
    let (alias, _) = f.preview(&["--out", "lib/math.fpas"], 1);
    assert_eq!(alias["diagnostics"][0]["code"], "artifact_alias");
    let (membership, _) = f.preview(&["--out", "lib/plan.fpas"], 1);
    assert_eq!(membership["diagnostics"][0]["code"], "artifact_membership");
    assert!(!f.0.join("lib/plan.fpas").exists());
    f.write("existing.json", "keep");
    let (existing, _) = f.preview(&["--out", "existing.json"], 1);
    assert_eq!(existing["diagnostics"][0]["code"], "artifact_exists");
    assert_eq!(
        fs::read_to_string(f.0.join("existing.json")).unwrap(),
        "keep"
    );
    let (mut report, _) = f.preview(&[], 0);
    report["result"]["plan"]["edits"][0]["new_text"] = json!("Tampered");
    f.write("bad.json", &serde_json::to_string(&report).unwrap());
    let (bad, _) = f.run(&["apply", "--plan", "bad.json"], 1);
    assert_eq!(bad["diagnostics"][0]["code"], "invalid_plan");
    assert_eq!(fs::read_to_string(f.0.join("app/main.fpas")).unwrap(), MAIN);
    assert_eq!(fs::read_to_string(f.0.join("lib/math.fpas")).unwrap(), UNIT);
}

#[test]
fn extract_and_import_plans_are_explicit_and_repeat_without_duplicate_edits() {
    let f = Fixture::new();
    f.write(
        "app/main.fpas",
        "program App;\nuses Demo.Math as M;\nbegin\n  const Value := (1 + 2) * 3;\nend.\n",
    );
    let (extracted, _) = f.run(
        &[
            "extract-variable",
            "--range",
            "app/main.fpas:4:18:4:25",
            "--context",
            "suite.fpasworkspace",
            "--std-lib",
            "std",
            "--out",
            "extract.json",
        ],
        0,
    );
    assert_eq!(
        extracted["result"]["plan"]["recipe"]["name"],
        "ExtractedValue"
    );
    f.run(&["apply", "--plan", "extract.json"], 0);
    assert_eq!(
        f.run(&["apply", "--plan", "extract.json"], 0).0["result"]["status"],
        "already_applied"
    );
    f.run(
        &[
            "organize-imports",
            "app/main.fpas",
            "--context",
            "suite.fpasworkspace",
            "--std-lib",
            "std",
            "--out",
            "imports.json",
        ],
        0,
    );
    f.run(&["apply", "--plan", "imports.json"], 0);
    assert_eq!(
        f.run(&["apply", "--plan", "imports.json"], 0).0["result"]["status"],
        "already_applied"
    );
    let (empty, _) = f.run(
        &[
            "organize-imports",
            "app/main.fpas",
            "--context",
            "suite.fpasworkspace",
            "--std-lib",
            "std",
            "--out",
            "empty.json",
        ],
        0,
    );
    assert_eq!(empty["result"]["status"], "no_change");
    assert_eq!(
        f.run(&["apply", "--plan", "empty.json"], 0).0["result"]["status"],
        "no_change"
    );
}

#[test]
fn source_permissions_prevent_all_replacements_before_staging_completes() {
    let f = Fixture::new();
    f.preview(&["--out", "rename.json"], 0);
    let path = f.0.join("lib/math.fpas");
    let permissions = fs::metadata(&path).unwrap().permissions();
    let mut readonly = permissions.clone();
    readonly.set_readonly(true);
    fs::set_permissions(&path, readonly).unwrap();
    let output = f.output(&["apply", "--plan", "rename.json"], &f.0);
    fs::set_permissions(&path, permissions).unwrap();
    let (failed, _) = decode(output, 3);
    assert_eq!(failed["result"]["publication"]["status"], "failed");
    assert!(
        failed["result"]["publication"]["paths"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["state"] == "unchanged")
    );
    assert_eq!(fs::read_to_string(f.0.join("app/main.fpas")).unwrap(), MAIN);
    assert_eq!(fs::read_to_string(path).unwrap(), UNIT);
}

#[cfg(windows)]
#[test]
fn a_real_later_replacement_failure_restores_the_earlier_file() {
    use std::os::windows::fs::OpenOptionsExt;
    let f = Fixture::new();
    f.preview(&["--out", "rename.json"], 0);
    let locked = fs::OpenOptions::new()
        .read(true)
        .share_mode(0x1 | 0x2)
        .open(f.0.join("lib/math.fpas"))
        .unwrap();
    let (failed, _) = f.run(&["apply", "--plan", "rename.json"], 3);
    drop(locked);
    assert_eq!(failed["result"]["publication"]["status"], "restored");
    let states = failed["result"]["publication"]["paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["state"].clone())
        .collect::<Vec<_>>();
    assert_eq!(states, vec![json!("restored"), json!("unchanged")]);
    assert_eq!(fs::read_to_string(f.0.join("app/main.fpas")).unwrap(), MAIN);
    assert_eq!(fs::read_to_string(f.0.join("lib/math.fpas")).unwrap(), UNIT);
    assert_eq!(
        fs::read_dir(f.0.join("app")).unwrap().count(),
        2,
        "no staging file survives"
    );
}
