//! Operation-specific and platform lifecycle checks share the real query fixture.

use super::*;

#[test]
fn definition_returns_source_names_for_aliases_members_self_and_callable_values() {
    let f = Fixture::new(
        "program App;\nuses Demo.Math as M;\nbegin\n  const Read := M.Answer;\n  discard Read();\nend.\n",
    );
    f.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    f.write("math.fpas","unit Demo.Math;\npublic function Answer(): integer;\nbegin\n  return 42;\nend function;\nend unit;\n");
    for (position, file, line, column) in [
        ("main.fpas:2:6", "math.fpas", 1, 6),
        ("main.fpas:4:17", "main.fpas", 2, 19),
        ("main.fpas:4:19", "math.fpas", 2, 17),
        ("main.fpas:5:11", "main.fpas", 4, 9),
        ("math.fpas:2:17", "math.fpas", 2, 17),
    ] {
        let r = f.run(&["definition", position], 0);
        assert_eq!(r["matches"].as_array().unwrap().len(), 1, "{r}");
        assert_eq!(r["matches"][0]["path"], file);
        assert_eq!(
            r["matches"][0]["range"]["start"],
            json!({"line":line,"column":column})
        );
        assert_eq!(r["matches"][0], r["target"]["declaration"]);
    }
    f.write("other.fpasprj","[project]\nname='other'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n");
    let ambiguous = f.run(&["definition", "main.fpas:5:11"], 1);
    assert_eq!(ambiguous["outcome"], "error");
    assert!(
        ambiguous["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["message"]
                .as_str()
                .is_some_and(|m| m.contains("multiple FPAS projects")))
    );
}

#[test]
fn builtin_without_source_is_distinct_from_missing_and_queries_never_execute() {
    let f = Fixture::new(
        "program T;\nuses Std.Fs as Fs;\nvar X: integer := 1;\nbegin\n  discard Fs.WriteText('executed','bad');\n  discard 'text'.Length();\nend.\n",
    );
    let refs = f.run(&["refs", "main.fpas:3:8"], 3);
    assert_eq!(refs["target"]["origin"], "builtin");
    assert!(refs["target"]["declaration"].is_null());
    assert_eq!(refs["coverage"]["reasons"], json!(["unsupported_target"]));
    let def = f.run(&["definition", "main.fpas:3:8"], 1);
    assert_eq!(def["outcome"], "not_found");
    assert_eq!(def["diagnostics"][0]["code"], "no_source_definition");
    let native = f.run(&["definition", "main.fpas:6:18"], 1);
    assert_eq!(native["target"]["origin"], "builtin");
    assert_eq!(native["target"]["qualified_name"], "string.Length");
    assert!(!f.0.join("executed").exists());
}

#[test]
fn checked_type_has_a_value_object_and_missing_metadata_is_incomplete() {
    let f = Fixture::new(
        "program T;\nbegin\n  const Count := 1;\n  discard '😀€'; discard Count + 2;\nend.\n",
    );
    let r = f.run(&["type", "main.fpas:3:9"], 0);
    assert_eq!(r["matches"], json!([]));
    assert_eq!(r["type"]["display"], "integer");
    assert_eq!(r["type"]["kind"], "integer");
    assert_eq!(
        r["type"]["location"]["range"]["start"],
        json!({"line":3,"column":9})
    );
    let string = f.run(&["type", "main.fpas:4:12"], 0);
    assert_eq!(string["type"]["display"], "string");
    let binary = f.run(&["type", "main.fpas:4:31"], 0);
    assert_eq!(
        binary["type"]["location"]["range"],
        json!({"start":{"line":4,"column":25},"end":{"line":4,"column":34}})
    );
    let absent = f.run(&["type", "main.fpas:5:1"], 1);
    assert_eq!(absent["outcome"], "not_found");
    f.write(
        "main.fpas",
        "program T; begin const Count := Missing; discard Count; end.",
    );
    let broken = f.run(&["type", "main.fpas:1:24"], 3);
    assert!(broken["type"].is_null());
    assert!(
        broken["coverage"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("type_unavailable"))
    );
    assert!(
        broken["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "FP3003")
    );
}

#[test]
fn locations_keep_unicode_scalars_utf8_bytes_and_all_line_endings() {
    let f = Fixture::new(
        "\u{feff}program T;\r\nbegin\r  const Count := 1;\n  discard '😀€'; discard Count;\rend.\r",
    );
    let r = f.run(&["refs", "main.fpas:3:9"], 0);
    let usage = &r["matches"][0];
    assert_eq!(
        usage["range"],
        json!({"start":{"line":4,"column":25},"end":{"line":4,"column":30}})
    );
    assert_eq!(usage["byte_span"], json!({"start":70,"length":5}));
    assert_eq!(usage["preview"]["text"], "  discard '😀€'; discard Count;");
    assert_eq!(usage["preview"]["column_start"], 1);
    let absolute = format!("{}:4:25", f.0.join("main.fpas").display());
    let r = f.run(&["definition", &absolute], 0);
    assert_eq!(
        r["matches"][0]["range"]["start"],
        json!({"line":3,"column":9})
    );
}

#[cfg(unix)]
#[test]
fn deadline_covers_blocked_source_reads_and_reaps_the_worker() {
    let f = Fixture::new(LOCAL);
    let fifo = f.0.join("blocked.fpas");
    nix::unistd::mkfifo(
        &fifo,
        nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
    )
    .unwrap();
    let started = std::time::Instant::now();
    let r = f.run(&["refs", "blocked.fpas:1:1", "--timeout", "1"], 3);
    assert!(started.elapsed() < std::time::Duration::from_secs(8));
    assert!(
        r["coverage"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("execution_timeout"))
    );
    assert!(r["target"].is_null());
}

#[test]
fn help_and_version_retain_plain_cli_output() {
    let f = Fixture::new(LOCAL);
    for args in [
        vec!["query", "--help"],
        vec!["query", "refs", "--help"],
        vec!["query", "definition", "--help"],
        vec!["query", "type", "--help"],
        vec!["query", "--version"],
    ] {
        let output = f.command().args(&args).output().unwrap();
        assert!(output.status.success(), "{args:?}");
        assert!(output.stderr.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(if args.contains(&"--version") {
                "fpas "
            } else {
                "Examples:"
            })
        );
    }
}

#[test]
fn imported_standard_parse_diagnostics_keep_their_file_in_partial_results() {
    let f = Fixture::new("program T; uses Std.Example as E; begin discard E.Read(); end.");
    f.write(
        "lib/stdlib.fpasprj",
        "[project]\nname='Std'\nkind='library'\n[sources]\ninclude=['*.fpas']\n",
    );
    f.write("lib/example.fpas", "unit Std.Example; public function Read(): integer; begin return 1 + ; end function; end unit;");
    let result = f.run(&["type", "main.fpas:1:49", "--std-lib", "lib"], 3);
    assert_eq!(result["outcome"], "incomplete");
    assert!(result["type"].is_null());
    assert!(
        result["diagnostics"].as_array().unwrap().iter().any(|d| {
            d["code"] == "FP2005" && d["source"] == "lib/example.fpas" && d["location"].is_object()
        }),
        "{result}"
    );
}
