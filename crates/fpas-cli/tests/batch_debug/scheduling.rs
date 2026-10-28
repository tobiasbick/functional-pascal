//! Real-process admission, repetition and honest seeded scheduling coverage.

use super::*;

const ORDERING: &str = r#"program Ordered;
uses Std.Tasks as Tasks, Std.Time as Time, Std.Console as Console;
procedure Produce(Q: channel of integer; Token: Tasks.CancellationToken; Value: integer);
begin
  const Ready := Tasks.CancellationCase(Token, procedure() begin end procedure);
  discard Tasks.Select([Ready]);
  discard Tasks.Send(Q, Value);
end procedure;
begin scope
  const Q: channel of integer := Tasks.CreateChannel(1);
  const Source := Tasks.CreateCancellationSource();
  const Token := Tasks.GetCancellationToken(Source);
  discard go Produce(Q, Token, 1);
  discard go Produce(Q, Token, 2);
  Time.Sleep(1000);
  discard Tasks.Cancel(Source);
  Console.WriteLn(Tasks.Receive(Q).Unwrap());
  Console.WriteLn(Tasks.Receive(Q).Unwrap());
end scope; end."#;

#[test]
fn seeded_runs_repeat_across_fresh_processes_and_explore_real_channel_order() {
    let f = Fixture::new(ORDERING);
    let mut outcomes = std::collections::BTreeSet::new();
    for seed in 0..16 {
        let seed = seed.to_string();
        let args = ["debug", "main.fpas", "--schedule-seed", &seed];
        let a = f.run(&args, 0);
        let b = f.run(&args, 0);
        assert_eq!(text(&a), text(&b));
        assert_eq!(a["schedule"], b["schedule"]);
        assert_eq!(a["schedule"]["status"], "controlled");
        assert_eq!(a["schedule"]["algorithm"], "seeded-rounds-v1");
        outcomes.insert(text(&a));
    }
    assert_eq!(outcomes.len(), 2);
}

#[test]
fn seed_boundaries_normalization_and_request_errors_are_explicit() {
    let f = Fixture::new(HELLO);
    for (seed, normalized) in [
        ("0", "0"),
        ("00042", "42"),
        ("18446744073709551615", "18446744073709551615"),
    ] {
        let r = f.run(&["debug", "main.fpas", "--schedule-seed", seed], 0);
        assert_eq!(r["schedule"]["seed"], normalized);
    }
    for seed in [
        "",
        "-1",
        "+1",
        " 1",
        "1 ",
        "1_000",
        "0x2a",
        "18446744073709551616",
    ] {
        let r = f.run(&["debug", "main.fpas", "--schedule-seed", seed], 1);
        assert_eq!(r["schedule"]["seed"], Value::Null);
    }
    for args in [
        vec!["--schedule-seed"],
        vec!["--schedule-seed", "1", "--schedule-seed", "2"],
        vec!["--schedule-seed", "1", "--protocol", "dap"],
        vec!["--schedule-seed", "1", "--protocol", "jsonl"],
    ] {
        let mut request = vec!["debug", "main.fpas"];
        request.extend(args);
        f.run(&request, 1);
    }
    let r = f.run(&["debug", "main.fpas"], 0);
    assert_eq!(r["schedule"]["status"], "not_requested");
    assert_eq!(r["schedule"]["clock"], "realtime");
}

#[test]
fn text_input_and_arguments_work_but_terminal_scripts_reject_before_initialization() {
    let f = Fixture::new(
        "program Input; uses Std.Console as Console, Std.Os.Args as Args; begin Console.WriteLn(Console.ReadLn().Unwrap()); Console.WriteLn(Args.ParamStr(0)); end.",
    );
    f.write("input.toml", "[[event]]\ntype = 'readln'\nline = 'fixed'\n");
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--schedule-seed",
            "42",
            "--script",
            "input.toml",
            "--",
            "--schedule-seed",
        ],
        0,
    );
    assert_eq!(text(&r), "fixed\n--schedule-seed\n");
    for script in [
        "[[event]]\ntype = 'readkey_chars'\nchars = 'a'\n",
        "[[event]]\ntype = 'console_key'\nkind = 'Escape'\n",
        "[[event]]\ntype = 'console_resize'\nwidth = 80\nheight = 24\n",
    ] {
        f.write("terminal.toml", script);
        let r = f.run(
            &[
                "debug",
                "main.fpas",
                "--schedule-seed",
                "42",
                "--script",
                "terminal.toml",
            ],
            1,
        );
        assert_eq!(r["schedule"]["status"], "not_started");
        assert_eq!(r["schedule"]["dispatches"], Value::Null);
        assert!(
            r["diagnostics"][0]["message"]
                .as_str()
                .unwrap()
                .contains("Seeded scheduling accepts"),
            "{r}"
        );
        assert!(text(&r).is_empty());
    }
}

#[test]
fn forbidden_file_effect_is_never_performed_even_through_a_callback() {
    let f = Fixture::new(
        "program Effect; uses Std.Io.Fs as Fs; function Bad(N: integer): integer; begin discard Fs.WriteText('forbidden', 'bad').Unwrap(); return N; end function; begin discard [1].Map(Bad); end.",
    );
    let r = f.run(&["debug", "main.fpas", "--schedule-seed", "42"], 2);
    assert!(!f.0.join("forbidden").exists());
    assert_eq!(r["diagnostics"][0]["code"], "FP5026");
    assert_eq!(r["schedule"]["status"], "rejected");
    assert_eq!(r["schedule"]["reasons"][0], "unsupported_effect");
}

#[test]
fn virtual_sleep_and_deadlock_have_real_metadata() {
    let f = Fixture::new("program Sleep; uses Std.Time as Time; begin Time.Sleep(1000000); end.");
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--schedule-seed",
            "42",
            "--timeout",
            "5",
        ],
        0,
    );
    assert!(
        r["schedule"]["virtual_time_millis"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            >= 1_000_000
    );
    f.write("main.fpas", "program Blocked; uses Std.Tasks as Tasks; begin const Q: channel of integer := Tasks.CreateChannel(1); discard Tasks.Receive(Q); end.");
    let r = f.run(&["debug", "main.fpas", "--schedule-seed", "42"], 3);
    assert_eq!(r["diagnostics"][0]["code"], "schedule_deadlock");
    assert_eq!(r["schedule"]["status"], "controlled");
    assert_eq!(r["tasks"][0]["state"], "waiting");
}

#[test]
fn setup_binding_and_instruction_limits_keep_seed_and_truthful_counters() {
    let f = Fixture::new(HELLO);
    for args in [
        vec!["debug", "missing.fpas", "--schedule-seed", "42"],
        vec![
            "debug",
            "main.fpas",
            "--schedule-seed",
            "42",
            "--break",
            "missing.fpas:1",
        ],
    ] {
        let r = f.run(&args, 1);
        assert_eq!(r["schedule"]["seed"], "42");
        assert_eq!(r["schedule"]["status"], "not_started");
        assert_eq!(r["schedule"]["dispatches"], Value::Null);
    }
    f.write(
        "main.fpas",
        "program Loop; begin while true do null; end while; end.",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--schedule-seed",
            "42",
            "--instruction-limit",
            "100",
        ],
        3,
    );
    assert_eq!(r["schedule"]["status"], "controlled");
    assert!(r["schedule"]["dispatches"].is_string());
}

#[cfg(unix)]
#[test]
fn preparation_watchdog_keeps_seed_without_inventing_worker_state() {
    let f = Fixture::new(HELLO);
    nix::unistd::mkfifo(
        &f.0.join("blocked.fpas"),
        nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
    )
    .unwrap();
    let r = f.run(
        &[
            "debug",
            "blocked.fpas",
            "--schedule-seed",
            "42",
            "--timeout",
            "1",
        ],
        3,
    );
    assert_eq!(r["schedule"]["seed"], "42");
    assert_eq!(r["schedule"]["status"], "interrupted");
    assert_eq!(r["schedule"]["dispatches"], Value::Null);
    assert_eq!(r["schedule"]["reasons"][0], "host_deadline");
}

#[test]
fn seeded_project_and_verified_image_share_the_same_execution() {
    let f = Fixture::new(ORDERING);
    f.write("app.fpasprj", "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['main.fpas']\n");
    let project = f.run(&["debug", "app.fpasprj", "--schedule-seed", "42"], 0);
    assert!(
        f.command()
            .args(["build", "app.fpasprj"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let image = f.run(
        &[
            "debug",
            "app.fpascp",
            "--source-root",
            ".",
            "--schedule-seed",
            "42",
        ],
        0,
    );
    assert_eq!(text(&project), text(&image));
    assert_eq!(project["schedule"], image["schedule"]);
}
