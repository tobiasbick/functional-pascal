//! Inspection through the isolated worker preserves exact excerpts and physical provenance.
use super::*;

#[test]
fn inspect_reports_exact_unicode_excerpts_and_explicit_text_omissions() {
    let source = format!(
        "program Main;\r\n// Attached documentation.\r\nfunction Answer(): integer;\r\nrequires true;\r\nbegin\r\n// {}\r\nreturn 42;\r\nend function;\r\nbegin end.",
        "😀".repeat(9000)
    );
    let f = Fixture::new(&source);
    f.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let report = f.run(
        &[
            "inspect",
            "--symbol",
            "Main.Answer",
            "--context",
            "app.fpasprj",
        ],
        0,
    );
    assert_eq!(report["result"]["kind"], "inspect");
    assert_eq!(report["coverage"]["analysis_complete"], true);
    assert!(report["target"]["target_ref"].is_object());
    assert_eq!(
        report["result"]["sections"][0]["documentation"],
        "Attached documentation."
    );
    let mut bytes = 0;
    for section in report["result"]["sections"].as_array().unwrap() {
        for excerpt in section["excerpts"].as_array().unwrap() {
            let span = &excerpt["location"]["byte_span"];
            let start = span["start"].as_u64().unwrap() as usize;
            let length = span["length"].as_u64().unwrap() as usize;
            let text = excerpt["text"].as_str().unwrap();
            assert_eq!(text, &source[start..start + length]);
            bytes += text.len();
        }
    }
    assert!(bytes <= 32768);
    assert_eq!(report["result"]["excerpt_bytes"], bytes);
    assert!(
        report["result"]["omissions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["reason"] == "excerpt_limit")
    );
}

#[test]
fn inspect_distinguishes_generated_apis_source_implementations_and_source_less_builtins() {
    let f = Fixture::new(
        "program Main; uses Std.Io.Fs as Fs, Std.Cli as Cli; begin discard 'test'.Length(); end.",
    );
    f.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let generated = f.run(
        &[
            "inspect",
            "--symbol",
            "Std.Io.Fs.ReadText",
            "--context",
            "app.fpasprj",
        ],
        0,
    );
    let sections = generated["result"]["sections"].as_array().unwrap();
    assert_eq!(sections[0]["origin"], "intrinsic_api");
    assert!(sections.iter().all(|s| s["kind"] != "implementation"));
    assert!(
        sections
            .iter()
            .any(|s| s["qualified_name"] == "Std.Io.Fs.FsError")
    );
    assert!(
        generated["result"]["omissions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["reason"] == "intrinsic_implementation")
    );
    let source = f.run(
        &[
            "inspect",
            "--symbol",
            "Std.Cli.Help",
            "--context",
            "app.fpasprj",
        ],
        0,
    );
    assert!(
        source["result"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["kind"] == "implementation" && s["origin"] == "standard_library")
    );
    let builtin = f.run(
        &[
            "inspect",
            "--symbol",
            "string.Length",
            "--context",
            "app.fpasprj",
        ],
        0,
    );
    assert_eq!(builtin["result"]["sections"][0]["location"], Value::Null);
    assert!(
        builtin["result"]["omissions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["reason"] == "no_source")
    );
    assert!(builtin["target"]["target_ref"].is_null());
}
