//! Deterministic regular-file registry, buffering and cancellation contracts.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "isolated file fixtures and assertions"
)]

mod extensions;

use std::cell::Cell;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::failure::Failure;
use super::registry::Files;
use super::state::MAX_BYTES;

#[test]
fn fieldless_mode_and_origin_enums_use_checked_integer_ordinals() {
    use crate::Value;
    for value in 0..=3 {
        assert_eq!(
            super::arguments::ordinal(&Value::Integer(value), true).expect("mode"),
            value
        );
    }
    for value in 0..=2 {
        assert_eq!(
            super::arguments::ordinal(&Value::Integer(value), false).expect("origin"),
            value
        );
    }
    assert!(matches!(
        super::arguments::ordinal(&Value::Integer(4), true),
        Err(Failure::Argument("Mode", _))
    ));
    assert!(matches!(
        super::arguments::ordinal(&Value::Integer(-1), false),
        Err(Failure::Argument("Origin", _))
    ));
    assert!(matches!(
        super::arguments::ordinal(&Value::Boolean(false), true),
        Err(Failure::Argument("Mode", _))
    ));
}

pub(super) struct Fixture(PathBuf);

impl Fixture {
    pub(super) fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        loop {
            let path = std::env::temp_dir().join(format!(
                "fpas-file-handles-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("temporary directory: {error}"),
            }
        }
    }

    pub(super) fn path(&self, name: &str) -> String {
        self.0.join(name).to_string_lossy().into_owned()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn modes_and_resource_admission_precede_destructive_open() {
    let fixture = Fixture::new();
    let path = fixture.path("kept");
    let missing = fixture.path("missing");
    fs::write(&path, b"kept").expect("fixture");
    let files = Files::with_limit(1);
    for mode in [0, 3] {
        assert!(matches!(
            files.open(&missing, mode, || false),
            Err(Failure::Io(_, _))
        ));
        assert!(!std::path::Path::new(&missing).exists());
    }
    assert!(matches!(
        files.open(&path, 4, || false),
        Err(Failure::Argument("Mode", _))
    ));
    assert_eq!(fs::read(&path).expect("preserved"), b"kept");
    let handle = files.open(&path, 0, || false).expect("read mode");
    assert!(matches!(
        files.write(handle, b"x", || false),
        Err(Failure::Argument("File", _))
    ));
    assert!(matches!(
        files.open(&path, 1, || false),
        Err(Failure::Limit(1))
    ));
    assert!(matches!(
        files.open(&missing, 1, || false),
        Err(Failure::Limit(1))
    ));
    assert!(!std::path::Path::new(&missing).exists());
    assert_eq!(fs::read(&path).expect("preserved at cap"), b"kept");
    files.close(handle, || false).expect("close read mode");
    let handle = files.open(&path, 1, || false).expect("explicit truncate");
    assert!(fs::read(&path).expect("truncated").is_empty());
    assert!(matches!(
        files.read(handle, 1, || false),
        Err(Failure::Argument("File", _))
    ));
    files.close(handle, || false).expect("close create");
    let append = files.open(&path, 2, || false).expect("append");
    assert_eq!(
        files.write(append, b"a", || false).expect("first append"),
        1
    );
    files
        .seek(append, 0, 0, || false)
        .expect("seek append handle");
    assert_eq!(
        files.write(append, b"b", || false).expect("second append"),
        1
    );
    files.flush(append, || false).expect("flush");
    assert_eq!(fs::read(&path).expect("append ignores seek"), b"ab");
}

#[test]
fn buffered_reads_writes_and_seek_use_the_logical_byte_position() {
    let fixture = Fixture::new();
    let path = fixture.path("mixed");
    fs::write(&path, b"one\nABCDE").expect("fixture");
    let files = Files::new();
    let handle = files.open(&path, 3, || false).expect("read-write");
    assert_eq!(
        files.read_line(handle, || false).expect("first line"),
        Some("one".into())
    );
    assert_eq!(files.tell(handle, || false).expect("after line"), 4);
    assert_eq!(
        files.read(handle, 2, || false).expect("buffered bytes"),
        b"AB"
    );
    assert_eq!(files.tell(handle, || false).expect("after bytes"), 6);
    assert_eq!(files.write(handle, b"zz", || false).expect("overwrite"), 2);
    assert_eq!(files.tell(handle, || false).expect("after overwrite"), 8);
    assert_eq!(
        fs::read(&path).expect("correct overwrite offset"),
        b"one\nABzzE"
    );
    assert_eq!(
        files.seek(handle, -1, 1, || false).expect("relative seek"),
        7
    );
    assert_eq!(files.read(handle, 10, || false).expect("remaining"), b"zE");
    assert_eq!(files.seek(handle, -1, 2, || false).expect("end seek"), 8);
    assert!(matches!(
        files.seek(handle, -1, 0, || false),
        Err(Failure::Argument("Offset", _))
    ));
    assert_eq!(
        files
            .tell(handle, || false)
            .expect("unchanged negative seek"),
        8
    );
    assert!(matches!(
        files.seek(handle, i64::MAX, 1, || false),
        Err(Failure::Limit(i64::MAX))
    ));
    assert_eq!(
        files
            .tell(handle, || false)
            .expect("unchanged overflow seek"),
        8
    );
    assert!(matches!(
        files.read(handle, 0, || true),
        Err(Failure::Argument("MaxBytes", _))
    ));
    assert!(matches!(
        files.read(handle, MAX_BYTES as i64 + 1, || true),
        Err(Failure::Limit(_))
    ));
    assert!(matches!(
        files.write(handle, &vec![0; MAX_BYTES + 1], || true),
        Err(Failure::Limit(_))
    ));
}

#[test]
fn line_endings_empty_lines_final_fragments_and_invalid_utf8_are_distinct() {
    let fixture = Fixture::new();
    let path = fixture.path("lines");
    fs::write(&path, b"\n\r\nx\ry\r\nlast\r").expect("fixture");
    let files = Files::new();
    let handle = files.open(&path, 0, || false).expect("open");
    for expected in ["", "", "x\ry", "last\r"] {
        assert_eq!(
            files.read_line(handle, || false).expect("line"),
            Some(expected.into())
        );
    }
    assert_eq!(files.read_line(handle, || false).expect("EOF"), None);
    assert!(
        files
            .read(handle, 1, || false)
            .expect("byte EOF")
            .is_empty()
    );
    files.close(handle, || false).expect("close");
    fs::write(&path, b"\xff\nok\n").expect("invalid UTF-8 fixture");
    let handle = files.open(&path, 0, || false).expect("reopen");
    assert!(matches!(
        files.read_line(handle, || false),
        Err(Failure::Encoding(_, _))
    ));
    assert_eq!(files.tell(handle, || false).expect("bad line consumed"), 2);
    assert_eq!(
        files.read_line(handle, || false).expect("next line"),
        Some("ok".into())
    );
}

#[test]
fn line_limit_excludes_terminators_and_overlong_prefixes_can_be_recovered() {
    let fixture = Fixture::new();
    let path = fixture.path("long");
    let files = Files::new();
    for ending in [b"\n".as_slice(), b"\r\n".as_slice(), b"".as_slice()] {
        let mut bytes = vec![b'x'; MAX_BYTES];
        bytes.extend(ending);
        fs::write(&path, bytes).expect("exact content boundary");
        let handle = files.open(&path, 0, || false).expect("open boundary");
        assert_eq!(
            files
                .read_line(handle, || false)
                .expect("boundary line")
                .expect("present")
                .len(),
            MAX_BYTES
        );
        assert_eq!(files.read_line(handle, || false).expect("EOF"), None);
        files.close(handle, || false).expect("close boundary");
    }
    let mut bytes = vec![b'x'; MAX_BYTES + 1];
    bytes.push(b'\n');
    fs::write(&path, bytes).expect("overlong fixture");
    let handle = files.open(&path, 0, || false).expect("open overlong");
    for _ in 0..2 {
        assert!(matches!(
            files.read_line(handle, || false),
            Err(Failure::Limit(_))
        ));
        assert_eq!(files.tell(handle, || false).expect("unconsumed prefix"), 0);
    }
    assert_eq!(files.read(handle, 1, || false).expect("recover byte"), b"x");
    assert_eq!(files.tell(handle, || false).expect("delivered byte"), 1);
    assert_eq!(
        files
            .read_line(handle, || false)
            .expect("recovered line")
            .expect("present")
            .len(),
        MAX_BYTES
    );
    files.seek(handle, 0, 0, || false).expect("reset");
    assert!(matches!(
        files.read_line(handle, || false),
        Err(Failure::Limit(_))
    ));
    files
        .seek(handle, 0, 2, || false)
        .expect("discard prefix with seek");
    assert_eq!(files.read_line(handle, || false).expect("EOF"), None);
}

#[test]
fn cancellation_retains_line_prefixes_and_never_replaces_successful_progress() {
    let fixture = Fixture::new();
    let path = fixture.path("cancel");
    let mut bytes = vec![b'x'; 20_000];
    bytes.push(b'\n');
    fs::write(&path, bytes).expect("fixture");
    let files = Files::new();
    let handle = files.open(&path, 3, || false).expect("read-write");
    let checks = Cell::new(0);
    assert!(matches!(
        files.read_line(handle, || {
            checks.set(checks.get() + 1);
            checks.get() >= 3
        }),
        Err(Failure::Cancelled)
    ));
    assert_eq!(
        files
            .tell(handle, || false)
            .expect("cancelled prefix retained"),
        0
    );
    assert_eq!(
        files
            .read_line(handle, || false)
            .expect("resume")
            .expect("line")
            .len(),
        20_000
    );
    assert!(matches!(
        files.write(handle, b"lost", || true),
        Err(Failure::Cancelled)
    ));
    assert_eq!(fs::metadata(&path).expect("unchanged length").len(), 20_001);
    let requested = Cell::new(false);
    assert_eq!(
        files
            .write(handle, b"ok", || {
                requested.set(true);
                false
            })
            .expect("successful write"),
        2
    );
    assert!(requested.get());
    assert_eq!(
        files.tell(handle, || false).expect("progress reported"),
        20_003
    );
    files.close(handle, || false).expect("close");
    assert!(matches!(
        files.read(handle, 1, || true),
        Err(Failure::Closed)
    ));
}

#[test]
fn aliases_close_synchronously_shutdown_cleans_up_and_registries_do_not_share_handles() {
    let fixture = Fixture::new();
    let path = fixture.path("lifetime");
    fs::write(&path, b"kept").expect("fixture");
    let first = Files::new();
    let second = Files::new();
    let handle = first.open(&path, 0, || false).expect("first registry");
    let other = second.open(&path, 0, || false).expect("second registry");
    assert_ne!(handle, other);
    assert!(matches!(
        second.tell(handle, || false),
        Err(Failure::Closed)
    ));
    let captured = first.entry(handle, &mut || false).expect("in-flight alias");
    first.close(handle, || false).expect("close");
    assert!(captured.state.lock().expect("state lock").is_none());
    assert!(matches!(first.tell(handle, || false), Err(Failure::Closed)));
    assert!(matches!(
        first.close(handle, || false),
        Err(Failure::Closed)
    ));
    second.shutdown();
    assert!(matches!(second.tell(other, || false), Err(Failure::Closed)));
    assert!(matches!(
        second.open(&path, 1, || false),
        Err(Failure::Closed)
    ));
    assert_eq!(
        fs::read(&path).expect("shutdown prevents truncate"),
        b"kept"
    );
    assert!(matches!(
        first.open(&fixture.path("."), 0, || false),
        Err(Failure::Argument("Path", _))
    ));
    #[cfg(unix)]
    assert!(matches!(
        first.open("/dev/null", 1, || false),
        Err(Failure::Argument("Path", _))
    ));
}

#[test]
fn concurrent_writes_to_one_handle_are_serialized_without_losing_bytes() {
    let fixture = Fixture::new();
    let path = fixture.path("concurrent");
    let files = Arc::new(Files::new());
    let handle = files.open(&path, 1, || false).expect("create");
    let mut workers = Vec::new();
    for byte in 0_u8..4 {
        let files = Arc::clone(&files);
        workers.push(std::thread::spawn(move || {
            for _ in 0..16 {
                let bytes = [byte; 16];
                let mut offset = 0;
                while offset < bytes.len() {
                    let count = files
                        .write(handle, &bytes[offset..], || false)
                        .expect("write");
                    assert!(count > 0);
                    offset += count;
                }
            }
        }));
    }
    for worker in workers {
        worker.join().expect("writer thread");
    }
    files.close(handle, || false).expect("close");
    let bytes = fs::read(&path).expect("written file");
    assert_eq!(bytes.len(), 1024);
    for byte in 0_u8..4 {
        assert_eq!(bytes.iter().filter(|value| **value == byte).count(), 256);
    }
}
