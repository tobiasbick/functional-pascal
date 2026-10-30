//! Linux fallback follows observed descendants across new process groups/sessions.
//!
//! PID descriptors pin verified PID/start-time identities before signalling. This
//! is best-effort tracking, not containment of unobserved daemon double-forks.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::os::fd::OwnedFd;
use std::time::{Duration, Instant};

use rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Identity {
    pid: u32,
    parent: u32,
    started: u64,
}

impl Identity {
    fn read(pid: u32) -> Option<Self> {
        let text = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        let fields: Vec<_> = text.rsplit_once(") ")?.1.split_whitespace().collect();
        Some(Self {
            pid,
            parent: fields.get(1)?.parse().ok()?,
            started: fields.get(19)?.parse().ok()?,
        })
    }

    fn same_process(self, other: Self) -> bool {
        self.pid == other.pid && self.started == other.started
    }

    fn pin(self) -> io::Result<Option<OwnedFd>> {
        let pid = i32::try_from(self.pid)
            .ok()
            .and_then(Pid::from_raw)
            .ok_or_else(|| io::Error::other("invalid descendant PID"))?;
        let handle = match pidfd_open(pid, PidfdFlags::empty()) {
            Ok(handle) => handle,
            Err(rustix::io::Errno::SRCH) => return Ok(None),
            Err(error) => return Err(io::Error::from(error)),
        };
        if Self::read(self.pid).is_some_and(|current| self.same_process(current)) {
            Ok(Some(handle))
        } else {
            Ok(None)
        }
    }
}

struct Tracked {
    identity: Identity,
    handle: OwnedFd,
}

pub(super) struct Tracker {
    root: Tracked,
    children: HashMap<u32, Tracked>,
    sampled: Instant,
}

impl Tracker {
    pub(super) fn new(pid: u32) -> io::Result<Self> {
        let identity = Identity::read(pid)
            .ok_or_else(|| io::Error::other("fixture leader identity unavailable"))?;
        let handle = identity
            .pin()?
            .ok_or_else(|| io::Error::other("fixture leader exited before tracking"))?;
        Ok(Self {
            root: Tracked { identity, handle },
            children: HashMap::new(),
            sampled: Instant::now() - Duration::from_secs(1),
        })
    }

    pub(super) fn sample(&mut self, force: bool) -> io::Result<()> {
        if !force && self.sampled.elapsed() < Duration::from_millis(10) {
            return Ok(());
        }
        self.sampled = Instant::now();
        let rows: Vec<_> = fs::read_dir("/proc")?
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
            .filter_map(Identity::read)
            .collect();
        let mut parents = HashMap::new();
        for row in &rows {
            if row.same_process(self.root.identity)
                || self
                    .children
                    .get(&row.pid)
                    .is_some_and(|known| row.same_process(known.identity))
            {
                parents.insert(row.pid, *row);
            }
        }
        loop {
            let mut added = false;
            for row in &rows {
                if parents.contains_key(&row.pid) {
                    continue;
                }
                let Some(parent) = parents.get(&row.parent).copied() else {
                    continue;
                };
                // /proc is not an atomic snapshot: a reused intermediate PID must
                // not admit an unrelated process into the fixture's owned tree.
                if !Identity::read(parent.pid).is_some_and(|current| parent.same_process(current)) {
                    continue;
                }
                if let Some(handle) = row.pin()? {
                    self.children.insert(
                        row.pid,
                        Tracked {
                            identity: *row,
                            handle,
                        },
                    );
                    parents.insert(row.pid, *row);
                    added = true;
                }
            }
            if !added {
                break;
            }
        }
        Ok(())
    }

    pub(super) fn kill(&mut self) -> io::Result<()> {
        let mut failure = self.sample(true).err();
        if let Err(error) = send(&self.root.handle, Signal::STOP) {
            failure.get_or_insert(error);
        }
        for child in self.children.values() {
            if let Err(error) = send(&child.handle, Signal::STOP) {
                failure.get_or_insert(error);
            }
        }
        // Catch children forked between the last sample and their parent's stop.
        if let Err(error) = self.sample(true) {
            failure.get_or_insert(error);
        }
        // A failed probe or stop must not abandon already stopped descendants.
        for child in self.children.values() {
            if let Err(error) = send(&child.handle, Signal::KILL) {
                failure.get_or_insert(error);
            }
        }
        failure.map_or(Ok(()), Err)
    }
}

fn send(handle: &OwnedFd, signal: Signal) -> io::Result<()> {
    match pidfd_send_signal(handle, signal) {
        Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
        Err(error) => Err(io::Error::from(error)),
    }
}
