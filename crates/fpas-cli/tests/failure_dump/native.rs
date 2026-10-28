//! A blocking native operation cannot be abandoned by report publication.

use super::*;

#[test]
#[ignore = "bounded native child, launched by blocked_owned_native_work_keeps_its_cleanup_barrier"]
fn blocked_process_fixture() {
    let root = PathBuf::from(
        std::env::var_os("FPAS_FAILURE_DUMP_TEST_GATE").expect("parent-controlled native gate"),
    );
    fs::write(root.join("native-ready"), "entered").expect("native receipt");
    let deadline = Instant::now() + Duration::from_secs(8);
    while !root.join("native-release").is_file() {
        assert!(Instant::now() < deadline, "native gate was never released");
        std::thread::sleep(Duration::from_millis(2));
    }
}

struct Release(PathBuf);
impl Drop for Release {
    fn drop(&mut self) {
        let _ = fs::write(self.0.join("native-release"), "release");
    }
}

#[test]
fn blocked_owned_native_work_keeps_its_cleanup_barrier() {
    let fixture = Fixture::new();
    let executable = std::env::current_exe()
        .expect("native helper executable")
        .to_str()
        .expect("test executable path")
        .replace('\'', "''");
    fixture.write("main.fpas", &format!("program Native; uses Std.Tasks as Tasks, Std.Os.Proc as Proc, Std.Io.Fs as Fs, Std.Time as Time, Std.Console as Console; begin const Group:=Tasks.CreateTaskGroup(); discard Tasks.StartTaskInGroup(Group, procedure(Token:Tasks.CancellationToken) begin discard Proc.RunCapture('{executable}', ['--ignored', '--exact', 'native::blocked_process_fixture']); end procedure); for I:integer:=1 to 6000 do if Fs.IsFile('native-ready') then Console.WriteLn('ready'); panic('owner failure'); end if; Time.Sleep(1); end for; panic('native helper timeout'); end."));
    let mut command = fixture.command();
    command
        .args(["run", "main.fpas", "--failure-dump", "report.json"])
        .env("FPAS_FAILURE_DUMP_TEST_GATE", &fixture.0);
    let mut child = Process::command(command);
    let release = Release(fixture.0.clone());
    child.ready();
    assert!(fixture.0.join("native-ready").is_file());
    assert!(
        !fixture.0.join("report.json").exists(),
        "required native ownership cleanup precedes terminal selection"
    );
    drop(release);
    let output = child.finish();
    exit(&output, 2);
    let report = fixture.report("report.json");
    assert_eq!(report["error"]["message"], "panic: owner failure");
    assert_eq!(report["error"]["task_id"], 0);
    assert_eq!(
        report["capture"]["total_tasks"], 1,
        "released child history is not reconstructed"
    );
    fixture.no_staging();
}
