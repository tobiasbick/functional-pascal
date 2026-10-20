//! Session containment includes nested VM process groups and orphaned children.

use std::collections::BTreeSet;
use std::io;
use std::os::unix::process::ExitStatusExt;
#[cfg(not(target_os = "linux"))]
use std::process::Stdio;
use std::process::{Child, Command, ExitStatus};
use std::time::{Duration, Instant};

use nix::sys::signal::{Signal, kill, killpg};
use nix::unistd::{Pid, getpgid, getsid};
use process_wrap::std::{CommandWrap, CommandWrapper, ProcessSession};
use rustix::process::{WaitId, WaitIdOptions, waitid};

#[cfg(target_os = "linux")]
mod linux;

pub(super) fn configure(command: &mut Command) -> io::Result<()> {
    // The public hook installs setsid in the dependency's pre-exec callback,
    // retaining std::process::Child and avoiding an additional wrapper chain.
    ProcessSession.pre_spawn(command, &CommandWrap::from(Command::new("")))
}

pub(super) fn try_wait_owned(child: &mut Child) -> io::Result<Option<ExitStatus>> {
    let pid = rustix::process::Pid::from_raw(pid(child)?.as_raw())
        .ok_or_else(|| io::Error::other("Invalid worker process ID"))?;
    let options = WaitIdOptions::EXITED | WaitIdOptions::NOWAIT | WaitIdOptions::NOHANG;
    loop {
        match waitid(WaitId::Pid(pid), options) {
            Err(error) if error == rustix::io::Errno::INTR => continue,
            Err(error) => return Err(error.into()),
            Ok(None) => return Ok(None),
            Ok(Some(status)) => {
                let raw = if let Some(code) = status.exit_status() {
                    code << 8
                } else if let Some(signal) = status.terminating_signal() {
                    signal | if status.dumped() { 0x80 } else { 0 }
                } else {
                    return Err(io::Error::other("Unexpected worker wait status"));
                };
                return Ok(Some(ExitStatus::from_raw(raw)));
            }
        }
    }
}

pub(super) fn terminate(child: &mut Child) -> io::Result<()> {
    let session = pid(child)?;
    // Stop the known child directly first. It may not yet have executed setsid;
    // signalling its inherited process group could stop the CLI itself.
    let _ = kill(session, Signal::SIGSTOP);
    let mut members = Vec::new();
    let mut previous = BTreeSet::new();
    let mut previous_quiescent = false;
    let mut failure = None;
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        let snapshot = match session_members(session) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                failure = Some(error);
                break;
            }
        };
        let groups = snapshot
            .iter()
            .map(|member| member.group)
            .collect::<BTreeSet<_>>();
        for group in groups {
            if let Err(error) = signal_owned_group(session, group, &snapshot, Signal::SIGSTOP) {
                failure.get_or_insert(error);
            }
        }
        let current = snapshot
            .iter()
            .map(|member| member.pid)
            .collect::<BTreeSet<_>>();
        let quiescent = snapshot.iter().all(|member| member.quiescent);
        let stable = current == previous && previous_quiescent && quiescent;
        members = snapshot;
        if stable {
            break;
        }
        if Instant::now() >= deadline {
            failure.get_or_insert_with(|| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Worker session did not stop before cleanup",
                )
            });
            break;
        }
        previous = current;
        previous_quiescent = quiescent;
        std::thread::yield_now();
    }
    // Kill nested groups before the worker group, retaining the session leader
    // until the final direct-child wait. Recheck ownership before every signal.
    let groups = members
        .iter()
        .map(|member| member.group)
        .collect::<BTreeSet<_>>();
    for group in groups.into_iter().filter(|group| *group != session) {
        if let Err(error) = signal_owned_group(session, group, &members, Signal::SIGKILL) {
            failure.get_or_insert(error);
        }
    }
    if let Err(error) = signal_owned_group(session, session, &members, Signal::SIGKILL) {
        failure.get_or_insert(error);
    }
    let _ = child.kill();
    if let Err(error) = child.wait() {
        failure.get_or_insert(error);
    }
    failure.map_or(Ok(()), Err)
}

struct Member {
    pid: Pid,
    group: Pid,
    quiescent: bool,
}

#[cfg(target_os = "linux")]
fn session_members(session: Pid) -> io::Result<Vec<Member>> {
    linux::session_members(session)
}

#[cfg(not(target_os = "linux"))]
fn session_members(session: Pid) -> io::Result<Vec<Member>> {
    // BSD/macOS and Linux disagree on the ps session-id column. Obtain only
    // portable process/state columns, then query session/group IDs safely.
    let executable = if std::path::Path::new("/bin/ps").is_file() {
        "/bin/ps"
    } else {
        "/usr/bin/ps"
    };
    let output = Command::new(executable)
        .args(["-e", "-o", "pid=", "-o", "stat="])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other("Cannot enumerate worker session members"));
    }
    let mut members = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut fields = line.split_whitespace();
        let Some(pid) = fields.next().and_then(|pid| pid.parse::<i32>().ok()) else {
            continue;
        };
        let Some(state) = fields
            .next()
            .and_then(|state| state.as_bytes().first())
            .copied()
        else {
            continue;
        };
        if pid <= 0 {
            continue;
        }
        let pid = Pid::from_raw(pid);
        if getsid(Some(pid)).ok() != Some(session) {
            continue;
        }
        let Ok(group) = getpgid(Some(pid)) else {
            continue;
        };
        if group.as_raw() <= 0 {
            continue;
        }
        members.push(Member {
            pid,
            group,
            quiescent: matches!(state, b'T' | b't' | b'Z'),
        });
    }
    Ok(members)
}

fn signal_owned_group(
    session: Pid,
    group: Pid,
    members: &[Member],
    signal: Signal,
) -> io::Result<()> {
    if members.iter().any(|member| {
        member.group == group
            && getsid(Some(member.pid)).ok() == Some(session)
            && getpgid(Some(member.pid)).ok() == Some(group)
    }) {
        match killpg(group, signal) {
            Ok(()) | Err(nix::errno::Errno::ESRCH) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn pid(child: &Child) -> io::Result<Pid> {
    i32::try_from(child.id())
        .ok()
        .filter(|pid| *pid > 0)
        .map(Pid::from_raw)
        .ok_or_else(|| io::Error::other("Invalid worker process ID"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_wait_preserves_native_status_and_leader_until_session_cleanup() {
        for (script, expected_code, expected_signal) in
            [("exit 7", Some(7), None), ("kill -TERM $$", None, Some(15))]
        {
            let mut command = Command::new("/bin/sh");
            command.args(["-c", script]);
            configure(&mut command).expect("session setup");
            let mut child = command.spawn().expect("worker");
            let leader = pid(&child).expect("worker PID");
            let wait_pid = rustix::process::Pid::from_raw(leader.as_raw()).expect("positive PID");
            waitid(
                WaitId::Pid(wait_pid),
                WaitIdOptions::EXITED | WaitIdOptions::NOWAIT,
            )
            .expect("waitable leader");
            let status = try_wait_owned(&mut child)
                .expect("peek status")
                .expect("exited");
            assert_eq!(status.code(), expected_code);
            assert_eq!(status.signal(), expected_signal);
            assert_eq!(
                getsid(Some(leader)).expect("leader remains waitable"),
                leader
            );
            terminate(&mut child).expect("session cleanup");
            assert_eq!(child.wait().expect("cached native status"), status);
        }
    }
}
