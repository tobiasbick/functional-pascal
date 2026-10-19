//! Exact anonymous callback selectors remain reusable across compiler processes.

use super::*;
use serde_json::json;

const COUNTERS: &str = r#"program Counters;
function Make(Seed: integer): function(): integer;
begin
  var Total := Seed;
  return function(): integer
  begin
    Total := Total + 1;
    return Total;
  end function;
end function;
begin
  const A := Make(0);
  const Copy := A;
  const B := Make(10);
  discard A();
  discard B();
  discard Copy();
end.
"#;

fn values(report: &Value, watch: usize) -> Vec<Value> {
    report["trace"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["watch_id"] == watch)
        .map(|event| event["new_value"]["scalar"].clone())
        .collect()
}

#[test]
fn stopped_callback_name_is_reusable_in_fresh_source_project_and_image_compilations() {
    let f = Fixture::new(COUNTERS);
    let stop = f.run(&["debug", "main.fpas", "--break", "main.fpas:8"], 0);
    assert_eq!(stop["outcome"], "breakpoint");
    let callback = stop["tasks"][0]["frames"][0]["function"]
        .as_str()
        .expect("exact callback name at stop");
    assert_eq!(callback, "make.$closure_1");
    let selector = format!("{callback}::Total");
    let r = f.run(&["debug", "main.fpas", "--trace", &selector], 0);
    assert_eq!(r["trace"]["complete"], true);
    assert_eq!(
        values(&r, 0),
        vec![
            json!("0"),
            json!("1"),
            json!("10"),
            json!("11"),
            json!("1"),
            json!("2")
        ]
    );
    assert_eq!(
        r["trace"],
        f.run(&["debug", "main.fpas", "--trace", &selector], 0)["trace"]
    );
    f.write("app.fpasprj", "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['main.fpas']\n");
    assert_eq!(
        r["trace"],
        f.run(&["debug", "app.fpasprj", "--trace", &selector], 0)["trace"]
    );
    let built = f.command().args(["build", "app.fpasprj"]).output().unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let image = f.run(
        &[
            "debug",
            "app.fpascp",
            "--source-root",
            ".",
            "--trace",
            &selector,
        ],
        0,
    );
    assert_eq!(r["trace"], image["trace"]);
}

#[test]
fn sibling_callbacks_and_shadowed_capture_sources_keep_separate_exact_identities() {
    let f = Fixture::new(
        "program T; function Check(Seed: integer): integer; begin const A := function(): integer begin return Seed; end function; begin const Seed := 9; const B := function(): integer begin return Seed; end function; return A() + B(); end; end function; begin if Check(7) <> 16 then panic('wrong captures'); end if; end.",
    );
    let args = [
        "debug",
        "main.fpas",
        "--trace",
        "Check.$closure_1::Seed",
        "--trace",
        "Check.$closure_2::Seed",
    ];
    let r = f.run(&args, 0);
    assert_eq!(values(&r, 0), vec![json!("7")]);
    assert_eq!(values(&r, 1), vec![json!("9")]);
    assert_ne!(
        r["trace"]["watches"][0]["identity"]["capture_source"],
        r["trace"]["watches"][1]["identity"]["capture_source"]
    );
    assert_eq!(r["trace"], f.run(&args, 0)["trace"]);
}

#[test]
fn same_named_factories_in_distinct_units_keep_qualified_callback_selectors() {
    let f = Fixture::new(
        "program T; uses LeftSide as Left, RightSide as Right; begin const A := Left.Make(7); const B := Right.Make(9); discard A(); discard B(); end.",
    );
    for unit in ["LeftSide", "RightSide"] {
        f.write(&format!("{unit}.fpas"), &format!("unit {unit}; public function Make(Seed: integer): function(): integer; begin return function(): integer begin return Seed; end function; end function; end unit;"));
    }
    f.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let selectors = [
        "--trace",
        "LeftSide.Make.$closure_1::Seed",
        "--trace",
        "RightSide.Make.$closure_1::Seed",
    ];
    let mut args = vec!["debug", "app.fpasprj"];
    args.extend(selectors);
    let r = f.run(&args, 0);
    assert_eq!(values(&r, 0), vec![json!("7")]);
    assert_eq!(values(&r, 1), vec![json!("9")]);
    // Require unchanged sidecars on the second build, not only equal reports.
    let mut previous = None;
    for _ in 0..2 {
        let built = f.command().args(["build", "app.fpasprj"]).output().unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        let stamps = ["LeftSide.fpascu", "RightSide.fpascu"]
            .map(|path| fs::metadata(f.0.join(path)).unwrap().modified().unwrap());
        if let Some(prior) = previous {
            assert_eq!(prior, stamps, "unchanged unit objects must be reused");
        }
        previous = Some(stamps);
        let mut args = vec!["debug", "app.fpascp", "--source-root", "."];
        args.extend(selectors);
        assert_eq!(r["trace"], f.run(&args, 0)["trace"]);
    }
}

#[test]
fn root_and_same_named_source_routine_have_unambiguous_callback_names() {
    let f = Fixture::new(
        "program dEmO; function Demo(Seed: integer): function(): integer; begin return function(): integer begin return Seed; end function; end function; begin const Seed := 9; const Root := function(): integer begin return Seed; end function; const Nested := Demo(7); if (Root() <> 9) or (Nested() <> 7) then panic('wrong callbacks'); end if; end.",
    );
    let selectors = [
        "--trace",
        "$entry_demo.$closure_1::Seed",
        "--trace",
        "Demo.$closure_1::Seed",
    ];
    let mut args = vec!["debug", "main.fpas"];
    args.extend(selectors);
    let r = f.run(&args, 0);
    assert_eq!(values(&r, 0), vec![json!("9")]);
    assert_eq!(values(&r, 1), vec![json!("7")]);
    assert_ne!(
        r["trace"]["watches"][0]["identity"]["function_id"],
        r["trace"]["watches"][1]["identity"]["function_id"]
    );
    f.write("app.fpasprj", "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['main.fpas']\n");
    let mut args = vec!["debug", "app.fpasprj"];
    args.extend(selectors);
    assert_eq!(r["trace"], f.run(&args, 0)["trace"]);
    let built = f.command().args(["build", "app.fpasprj"]).output().unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let mut args = vec!["debug", "app.fpascp", "--source-root", "."];
    args.extend(selectors);
    assert_eq!(r["trace"], f.run(&args, 0)["trace"]);
}
