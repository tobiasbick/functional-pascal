//! Private IPC and parent-owned deadlines shared by debug and source-query workers.

mod files;
pub(crate) use files::{Files, Request, read_bounded};
use std::{
    process::Child,
    time::{Duration, Instant},
};

/// Waits for atomic publication, requests cancellation and always reaps the owned tree.
pub(crate) fn wait(
    child: &mut Child,
    files: &Files,
    deadline: Instant,
    overflow: impl Fn() -> bool,
) -> (Result<serde_json::Value, String>, Option<&'static str>) {
    let mut ended = None;
    let mut grace_deadline = None;
    let result = loop {
        let overflow = overflow();
        if ended.is_none() && (overflow || Instant::now() >= deadline) {
            ended = Some(if overflow {
                "output_limit"
            } else {
                "execution_timeout"
            });
            let _ = std::fs::write(files.cancel(), []);
            grace_deadline = Some(Instant::now() + Duration::from_millis(250));
        }
        if files.result().is_file() {
            break files.read_result();
        }
        if grace_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            break Err("Worker exceeded its deadline or output limit.".into());
        }
        match child.try_wait() {
            Ok(Some(_)) => break Err("Worker terminated before publishing a result.".into()),
            Err(error) => break Err(error.to_string()),
            Ok(None) => {}
        }
        std::thread::sleep(Duration::from_millis(3));
    };
    crate::process_tree::terminate(child);
    (result, ended)
}
