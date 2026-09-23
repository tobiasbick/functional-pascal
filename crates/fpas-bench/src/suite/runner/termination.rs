//! Bounded process-tree termination and direct-child fallback.

use std::io;
use std::time::{Duration, Instant};

use process_wrap::std::ChildWrapper;

const REAP_TIMEOUT: Duration = Duration::from_secs(2);

/// Attempts tree termination, then polls rather than blocking on a failed kill.
pub(super) fn terminate_process_tree(
    child: &mut dyn ChildWrapper,
) -> (Option<String>, Option<io::Error>) {
    let mut termination_error = child.start_kill().err().map(|error| error.to_string());
    if let Some(message) = &mut termination_error {
        // Bypass this tree wrapper; the standard wrappers own the direct child underneath.
        let fallback = child.inner_mut().start_kill();
        if let Err(error) = fallback {
            message.push_str(&format!("; direct kill failed: {error}"));
        }
    }
    let deadline = Instant::now() + REAP_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return (termination_error, None),
            Err(error) => return (termination_error, Some(error)),
            Ok(None) if Instant::now() >= deadline => {
                return (
                    termination_error,
                    Some(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "process did not exit within the cleanup deadline",
                    )),
                );
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}
