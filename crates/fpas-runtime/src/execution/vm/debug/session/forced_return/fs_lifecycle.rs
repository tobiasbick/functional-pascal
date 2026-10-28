//! Forced root completion releases files without depending on debugger/host Arc destruction.

#![allow(
    clippy::expect_used,
    reason = "forced-completion fixtures inspect concrete source state"
)]

use super::*;
use crate::{FunctionId, VerifiedExecutable};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;

struct InputFile(PathBuf);

impl InputFile {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "fpas-debug-file-lifecycle-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, [42]).expect("source input file");
        Self(path)
    }
}

impl Drop for InputFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn compile(input: &InputFile, recover_failure: bool) -> VerifiedExecutable {
    let path = input.0.to_string_lossy().replace('\'', "''");
    let ending = if recover_failure {
        "panic('root failure');"
    } else {
        "Pause();"
    };
    let source = format!(
        r#"program DebugFileLifecycle;
uses Std.Fs as Fs;
var Saved: option of Fs.File := None;
procedure Pause();
begin panic('forced completion must skip this'); end procedure;
function Closed(File: Fs.File): boolean;
begin
  return (Fs.Tell(File) is Error(Fs.FsError.Closed('Std.Fs.Tell'))) and
    (Fs.Read(File, 1) is Error(Fs.FsError.Closed('Std.Fs.Read')));
end function;
begin
  Saved := Some(Fs.Open('{path}', Fs.FileMode.Read).Unwrap());
  {ending}
end."#
    );
    let (program, errors) = fpas_parser::parse(&source);
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program).expect("compile real debugger file source")
}

#[test]
fn forcing_or_recovering_main_entry_closes_live_files_before_session_drop() {
    for recover_failure in [false, true] {
        let input = InputFile::new();
        let mut session =
            DebugSession::new(compile(&input, recover_failure)).expect("debug session");
        if !recover_failure {
            session
                .replace_function_breakpoints(vec![FunctionBreakpoint {
                    name: "Pause".into(),
                }])
                .expect("pause after open");
        }
        assert!(matches!(
            session
                .continue_execution()
                .expect("stop after source open"),
            DebugRunResult::Stopped(_)
        ));
        assert_eq!(
            session.state(),
            if recover_failure {
                DebugSessionState::Failed
            } else {
                DebugSessionState::Stopped
            }
        );
        // Preserve the shared registry, layouts, globals and the actual source-opened handle.
        let worker = session.runtime.worker(0).expect("inspectable root");
        let executable = Arc::clone(&worker.executable);
        let layouts = Arc::clone(&worker.layouts);
        let globals = Arc::clone(&worker.globals);
        let hosted = Arc::clone(&worker.hosted);
        let image = executable.executable();
        let saved = image
            .globals
            .iter()
            .position(|global| {
                image.strings.get(global.name).is_some_and(|name| {
                    name.rsplit('.')
                        .next()
                        .is_some_and(|leaf| leaf.eq_ignore_ascii_case("Saved"))
                })
            })
            .expect("saved source handle");
        let file = match globals.read().expect("source globals")[saved].as_ref() {
            Some(Value::OptionSome(file)) => file.as_ref().clone(),
            other => panic!("file was not opened before stop: {other:?}"),
        };
        let target = image
            .functions
            .iter()
            .position(|function| {
                image.strings.get(function.name).is_some_and(|name| {
                    name.rsplit('.')
                        .next()
                        .is_some_and(|leaf| leaf.eq_ignore_ascii_case("Closed"))
                })
            })
            .expect("typed closure probe");
        let frame = session
            .stack(0, 8)
            .expect("root stack")
            .items
            .last()
            .expect("entry frame")
            .id;
        let completed = session
            .force_return(frame, None)
            .expect("complete or recover root entry");
        assert!(completed.terminated);
        assert_eq!(session.state(), DebugSessionState::Terminated);
        let outcome = Worker::for_function_with_state(
            executable,
            FunctionId::new(u16::try_from(target).expect("function index")),
            vec![file],
            globals,
            layouts,
            hosted,
        )
        .expect("shared-state probe")
        .run()
        .expect("typed closed outcomes");
        assert_eq!(outcome.value, Value::Boolean(true));
        drop(session);
    }
}
