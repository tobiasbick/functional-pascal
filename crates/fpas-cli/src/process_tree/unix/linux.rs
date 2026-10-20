//! Linux process snapshots avoid external commands and preserve arbitrary comm bytes.

use std::{fs, io};

use nix::unistd::{Pid, getsid};

use super::Member;

pub(super) fn session_members(session: Pid) -> io::Result<Vec<Member>> {
    let mut members = Vec::new();
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<i32>().ok())
        else {
            continue;
        };
        if pid <= 0 {
            continue;
        }
        if getsid(Some(Pid::from_raw(pid))).ok() != Some(session) {
            continue;
        }
        let bytes = match fs::read(entry.path().join("stat")) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) if error.raw_os_error() == Some(nix::libc::ESRCH) => continue,
            Err(error) => return Err(error),
        };
        if let Some(mut member) = parse_member(pid, &bytes, session) {
            // A stopped process leader can still have a thread finishing a
            // fork. Confirm every thread is quiescent before the stable pass.
            if member.quiescent {
                member.quiescent = tasks_quiescent(&entry.path())?;
            }
            members.push(member);
        }
    }
    Ok(members)
}

fn tasks_quiescent(process: &std::path::Path) -> io::Result<bool> {
    let mut observed = false;
    let tasks = match fs::read_dir(process.join("task")) {
        Ok(tasks) => tasks,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(true),
        Err(error) => return Err(error),
    };
    for entry in tasks {
        let bytes = match fs::read(entry?.path().join("stat")) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) if error.raw_os_error() == Some(nix::libc::ESRCH) => continue,
            Err(error) => return Err(error),
        };
        let Some(end) = bytes.iter().rposition(|byte| *byte == b')') else {
            return Ok(false);
        };
        let state = bytes
            .get(end + 1..)
            .and_then(|bytes| bytes.iter().find(|byte| !byte.is_ascii_whitespace()));
        if !matches!(state, Some(b'T' | b't' | b'Z')) {
            return Ok(false);
        }
        observed = true;
    }
    Ok(observed)
}

fn parse_member(pid: i32, bytes: &[u8], session: Pid) -> Option<Member> {
    // comm is parenthesized and may itself contain spaces, ')' or non-UTF8.
    let end = bytes.iter().rposition(|byte| *byte == b')')?;
    let mut fields = bytes
        .get(end + 1..)?
        .split(|byte| byte.is_ascii_whitespace())
        .filter(|field| !field.is_empty());
    let state = *fields.next()?.first()?;
    fields.next()?; // Parent PID is unrelated to ownership after reparenting.
    let group = integer(fields.next()?)?;
    if group <= 0 || integer(fields.next()?)? != session.as_raw() {
        return None;
    }
    Some(Member {
        pid: Pid::from_raw(pid),
        group: Pid::from_raw(group),
        quiescent: matches!(state, b'T' | b't' | b'Z'),
    })
}

fn integer(bytes: &[u8]) -> Option<i32> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_session_parser_handles_embedded_delimiters_and_excludes_foreign_sessions() {
        let bytes = b"21 (arbitrary ) \xff name) T 1 20 10 0 -1 0";
        let member = parse_member(21, bytes, Pid::from_raw(10)).expect("owned session");
        assert_eq!(member.pid, Pid::from_raw(21));
        assert_eq!(member.group, Pid::from_raw(20));
        assert!(member.quiescent);
        assert!(parse_member(21, bytes, Pid::from_raw(11)).is_none());
        assert!(
            !parse_member(21, b"21 (fork) D 1 20 10", Pid::from_raw(10))
                .expect("owned sleeper")
                .quiescent
        );
    }
}
