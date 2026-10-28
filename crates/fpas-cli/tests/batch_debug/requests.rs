use super::*;

#[test]
fn invalid_requests_always_get_one_json_object() {
    let f = Fixture::new(HELLO);
    for args in [
        vec!["--unknown"],
        vec!["--timeout", "0"],
        vec!["--output-limit", "1048577"],
        vec!["--protocol", "batch"],
        vec!["--task", "-1"],
        vec!["--break", "main.fpas:0"],
        vec!["--break", "absent.fpas:1"],
        vec!["--break", "main.fpas:999"],
        vec!["--print", "Total,"],
        vec!["--commands", "commands.jsonl"],
        vec!["--script", "absent.toml"],
    ] {
        let mut request = vec!["debug", "main.fpas"];
        request.extend(args);
        let r = f.run(&request, 1);
        assert_eq!(r["outcome"], "error");
    }
    f.run(&["debug", "absent.fpas"], 1);
}
#[test]
fn calls_are_rejected_before_any_bytecode_runs() {
    let f = Fixture::new(
        "program Guard; uses Std.Io.Fs as Fs; function Danger(): integer; begin return 42; end function; begin discard Fs.WriteText('executed', 'bad').Unwrap(); end.",
    );
    let r = f.run(&["debug", "main.fpas", "--print", "Danger()"], 1);
    assert_eq!(r["diagnostics"][0]["code"], "expression_syntax");
    assert!(!f.0.join("executed").exists());
}
#[test]
fn a_missed_breakpoint_differs_from_unbound_and_duplicates_are_deduplicated() {
    let f = Fixture::new(
        "program Miss;\nfunction Never(): integer;\nbegin\n return 1;\nend function;\nbegin\nend.\n",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--break",
            "main.fpas:4",
            "--break",
            "main.fpas:4",
        ],
        3,
    );
    assert_eq!(r["outcome"], "completed");
    assert_eq!(r["diagnostics"][0]["code"], "breakpoint_not_hit");
}
#[test]
fn script_line_and_key_queues_match_the_test_runner() {
    let f = Fixture::new(
        "program Input; uses Std.Console as Console; begin Console.WriteLn(Console.ReadLn().Unwrap()); Console.WriteText(Console.ReadKey()); Console.WriteText(Console.ReadKey()); end.",
    );
    f.write("input.toml", "[[event]]\ntype = 'readln'\nline = 'line'\n[[event]]\ntype = 'readkey_chars'\nchars = 'ab'\n");
    let r = f.run(&["debug", "main.fpas", "--script", "input.toml"], 0);
    assert_eq!(text(&r), "line\nab");
    f.run(&["debug", "main.fpas"], 2);
    f.write("input.toml", "bad toml!");
    f.run(&["debug", "main.fpas", "--script", "input.toml"], 1);
}
#[test]
fn project_discovery_workspace_and_verified_image_share_target_preparation() {
    let f = Fixture::new(HELLO);
    f.write("app.fpasprj", "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['main.fpas']\n");
    for args in [vec!["debug"], vec!["debug", "app.fpasprj"]] {
        assert_eq!(f.run(&args, 0)["outcome"], "completed");
    }
    let build = f.command().args(["build", "app.fpasprj"]).output().unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    f.run(&["debug", "app.fpascp", "--source-root", "."], 0);
    f.write("main.fpas", "program Changed; begin end.");
    let r = f.run(&["debug", "app.fpascp", "--source-root", "."], 1);
    assert!(
        r["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("stale")
    );
}

#[test]
fn compile_diagnostics_remain_structured_and_single_options_are_strict() {
    let f = Fixture::new("program Invalid; begin Missing(); end.");
    let r = f.run(&["debug", "main.fpas"], 1);
    assert!(
        r["diagnostics"][0]["code"]
            .as_str()
            .unwrap()
            .starts_with("FP")
    );
    f.write("main.fpas", HELLO);
    for flag in ["--timeout", "--instruction-limit", "--output-limit"] {
        f.run(&["debug", "main.fpas", flag, "100", flag, "100"], 1);
    }
    let mut args = vec!["debug", "main.fpas"];
    for _ in 0..65 {
        args.extend(["--print", "Total"]);
    }
    f.run(&args, 1);
    f.write("large.toml", &"x".repeat(1048577));
    f.run(&["debug", "main.fpas", "--script", "large.toml"], 1);
}

#[test]
fn setup_errors_keep_configured_limits_and_workspace_targets_use_discovery() {
    let f = Fixture::new(HELLO);
    let r = f.run(&["debug", "missing.fpas", "--instruction-limit", "17"], 1);
    assert_eq!(r["limits"]["instruction_limit"], 17);
    f.write("app.fpasprj", "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['main.fpas']\n");
    f.write(
        "suite.fpasworkspace",
        "[workspace]\nname = 'suite'\nmembers = ['app.fpasprj']\n",
    );
    for args in [vec!["debug"], vec!["debug", "suite.fpasworkspace"]] {
        assert_eq!(f.run(&args, 0)["outcome"], "completed");
    }
}

#[test]
fn version_and_help_remain_ordinary_cli_requests() {
    let f = Fixture::new(HELLO);
    for flag in ["--version", "--help"] {
        let r = f.command().args(["debug", flag]).output().unwrap();
        assert!(r.status.success());
        assert!(r.stderr.is_empty());
        assert!(
            std::str::from_utf8(&r.stdout)
                .unwrap()
                .contains(if flag == "--version" {
                    "fpas "
                } else {
                    "Usage:"
                })
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn escaped_argv_transport_has_its_own_bound_from_scripted_input() {
    let f = Fixture::new(HELLO);
    let expression = format!("'{}'", "\u{1}".repeat(4094));
    let mut args = vec!["debug", "main.fpas"];
    for _ in 0..64 {
        args.extend(["--print", &expression]);
    }
    let r = f.run(&args, 0);
    assert_eq!(r["expressions"].as_array().unwrap().len(), 64);
    assert_eq!(
        r["expressions"][63]["value"]["scalar"]
            .as_str()
            .unwrap()
            .len(),
        4094
    );
}
