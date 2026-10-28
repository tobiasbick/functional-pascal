use super::*;
use std::{
    sync::{Arc, atomic::AtomicBool},
    thread::JoinHandle,
};

#[test]
fn publication_failure_adds_fp4139_after_the_original_error_in_both_formats() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", "program Main; uses Std.Console as Console; begin Console.WriteLn('out'); panic('original'); end.");
    fixture.write("parent-file", "preserved");
    for format in ["text", "json"] {
        let output = fixture.run(&[
            "run",
            "main.fpas",
            "--failure-dump",
            "parent-file/report.json",
            "--diagnostics",
            format,
        ]);
        exit(&output, 2);
        assert_eq!(output.stdout, b"out\n");
        if format == "json" {
            let records = records(&output);
            assert_eq!(records.len(), 2);
            assert_eq!(records[0]["message"], "panic: original");
            assert_eq!(records[1]["code"], "FP4139");
        } else {
            let text = String::from_utf8_lossy(&output.stderr);
            assert!(
                text.find("panic: original").expect("original diagnostic")
                    < text.find("FP4139").expect("publication diagnostic")
            );
        }
    }
    assert_eq!(
        fs::read_to_string(fixture.0.join("parent-file")).expect("original"),
        "preserved"
    );
    fixture.no_staging();
}

#[test]
fn a_destination_created_after_validation_is_preserved_and_staging_is_removed() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", "program Race; uses Std.Console as Console, Std.Io.Fs as Fs, Std.Time as Time; begin Console.WriteLn('ready'); for I:integer:=1 to 6000 do if Fs.IsFile('release') then panic('raced'); end if; Time.Sleep(1); end for; panic('barrier timeout'); end.");
    let mut child = Process::spawn(
        &fixture,
        &[
            "run",
            "main.fpas",
            "--failure-dump",
            "report.json",
            "--diagnostics",
            "json",
        ],
    );
    child.ready();
    fixture.write("report.json", "competitor data");
    fixture.write("release", "go");
    let output = child.finish();
    exit(&output, 2);
    let records = records(&output);
    assert_eq!(records[0]["message"], "panic: raced");
    assert_eq!(records[1]["code"], "FP4139");
    assert_eq!(
        fs::read_to_string(fixture.0.join("report.json")).expect("competitor file"),
        "competitor data"
    );
    fixture.no_staging();
}

struct Monitor {
    stop: Arc<AtomicBool>,
    seen: Arc<std::sync::atomic::AtomicUsize>,
    thread: Option<JoinHandle<Result<usize, String>>>,
}
impl Monitor {
    fn start(path: PathBuf) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let finished = Arc::clone(&stop);
        let seen = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let published = Arc::clone(&seen);
        let thread = std::thread::spawn(move || {
            let mut observed = 0;
            while !finished.load(Ordering::Acquire) {
                match fs::read(&path) {
                    Ok(bytes) => {
                        serde_json::from_slice::<Value>(&bytes)
                            .map_err(|error| format!("partial report became visible: {error}"))?;
                        observed += 1;
                        published.store(observed, Ordering::Release);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.to_string()),
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            Ok(observed)
        });
        Self {
            stop,
            seen,
            thread: Some(thread),
        }
    }
    fn finish(mut self) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while self.seen.load(Ordering::Acquire) == 0
            && Instant::now() < deadline
            && !self.thread.as_ref().expect("monitor").is_finished()
        {
            std::thread::sleep(Duration::from_millis(1));
        }
        self.stop.store(true, Ordering::Release);
        let count = self
            .thread
            .take()
            .expect("monitor")
            .join()
            .expect("monitor thread")
            .expect("atomic observations");
        assert!(
            count > 0,
            "the monitor must actually read a published report"
        );
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[test]
fn concurrent_failure_writers_publish_exactly_one_complete_document() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", "program Writers; uses Std.Console as Console, Std.Io.Fs as Fs, Std.Time as Time, Std.Os.Args as Args; begin Console.WriteLn('ready'); for I:integer:=1 to 6000 do if Fs.IsFile('release') then var Items:array of string:=[]; for J:integer:=1 to 256 do Items.PushInPlace((#0).RepeatStr(4096)); end for; panic(Args.ParamStr(0)); end if; Time.Sleep(1); end for; panic('barrier timeout'); end.");
    let args = [
        "run",
        "main.fpas",
        "--failure-dump",
        "report.json",
        "--diagnostics",
        "json",
        "--",
    ];
    let mut one_args = args.to_vec();
    one_args.push("one");
    let mut two_args = args.to_vec();
    two_args.push("two");
    let mut one = Process::spawn(&fixture, &one_args);
    let mut two = Process::spawn(&fixture, &two_args);
    one.ready();
    two.ready();
    let monitor = Monitor::start(fixture.0.join("report.json"));
    fixture.write("release", "go");
    let outputs = [one.finish(), two.finish()];
    monitor.finish();
    for output in &outputs {
        exit(output, 2);
    }
    let publication_errors = outputs
        .iter()
        .flat_map(records)
        .filter(|record| record["code"] == "FP4139")
        .count();
    assert_eq!(
        publication_errors, 1,
        "one writer must lose the atomic new-file race"
    );
    let report = fixture.report("report.json");
    assert!(matches!(
        report["error"]["message"].as_str(),
        Some("panic: one" | "panic: two")
    ));
    fixture.no_staging();
}
