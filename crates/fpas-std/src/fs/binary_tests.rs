//! Binary identity, bounded reads and preflight ordering at the filesystem boundary.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "filesystem fixtures and typed boundary assertions must succeed"
)]

use std::fs;
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};

use fpas_bytecode::{FsIntrinsic, Intrinsic, SourceLocation, Value};

use super::binary::{MAX_BYTES, read_bytes, read_limited, write_bytes};
use super::failure::Failure;

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        Self(PathBuf::from(
            super::temp::create_temp_dir().expect("isolated temporary directory"),
        ))
    }

    fn path(&self, name: &str) -> String {
        self.0.join(name).to_string_lossy().into_owned()
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

fn invoke(operation: FsIntrinsic, path: &str, bytes: Option<Vec<Value>>) -> Value {
    let mut arguments = vec![Value::Str(path.into())];
    if let Some(bytes) = bytes {
        arguments.push(Value::Array(bytes.into()));
    }
    crate::execute_test_intrinsic(Intrinsic::Fs(operation), &mut arguments, location())
        .expect("valid intrinsic arguments");
    arguments.pop().expect("one returned value")
}

fn assert_error(value: &Value, variant: &str, fields: &[Value]) {
    let Value::ResultError(error) = value else {
        panic!("expected an error result");
    };
    let Value::Enum(error) = error.as_ref() else {
        panic!("expected a typed filesystem error");
    };
    assert_eq!(error.body().layout.type_name, "Std.Fs.FsError");
    assert_eq!(error.body().layout.variant, variant);
    assert_eq!(error.body().values.as_slice(), fields);
}

#[test]
fn every_byte_round_trips_without_utf8_conversion_and_empty_writes_truncate() {
    let root = Directory::new();
    let path = root.path("all-bytes.bin");
    let bytes: Vec<u8> = (0..=255).collect();
    let values: Vec<Value> = bytes
        .iter()
        .map(|byte| Value::Integer(i64::from(*byte)))
        .collect();
    assert_eq!(
        invoke(FsIntrinsic::WriteBytes, &path, Some(values.clone())),
        Value::result_ok(Value::Boolean(true))
    );
    assert_eq!(fs::read(&path).expect("written binary file"), bytes);
    assert_eq!(
        invoke(FsIntrinsic::ReadBytes, &path, None),
        Value::result_ok(Value::Array(values.into()))
    );
    for replacement in [vec![0_u8, 255, 128], vec![]] {
        let values: Vec<Value> = replacement
            .iter()
            .map(|byte| Value::Integer(i64::from(*byte)))
            .collect();
        assert_eq!(
            invoke(FsIntrinsic::WriteBytes, &path, Some(values.clone())),
            Value::result_ok(Value::Boolean(true))
        );
        assert_eq!(fs::read(&path).expect("truncated binary file"), replacement);
        assert_eq!(
            invoke(FsIntrinsic::ReadBytes, &path, None),
            Value::result_ok(Value::Array(values.into()))
        );
    }
}

#[test]
fn exact_byte_limit_is_accepted_and_oversized_metadata_is_rejected() {
    let root = Directory::new();
    let path = root.path("boundary.bin");
    let values = vec![Value::Integer(255); MAX_BYTES];
    write_bytes(&path, &values, location())
        .expect("integer array")
        .expect("exact boundary write");
    let bytes = read_bytes(&path).expect("exact boundary read");
    assert_eq!(bytes.len(), MAX_BYTES);
    assert!(bytes.iter().all(|byte| *byte == 255));
    let file = fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("boundary file");
    file.set_len((MAX_BYTES + 1) as u64)
        .expect("oversized fixture");
    drop(file);
    assert_error(
        &invoke(FsIntrinsic::ReadBytes, &path, None),
        "LimitExceeded",
        &[
            Value::Str("Std.Fs.ReadBytes".into()),
            Value::Integer(MAX_BYTES as i64),
        ],
    );
}

#[test]
fn untrusted_size_reads_stop_after_one_over_limit_byte() {
    let mut source = Cursor::new(vec![0_u8; MAX_BYTES + 100]);
    assert!(matches!(
        read_limited(&mut source),
        Err(Failure::LimitExceeded("Std.Fs.ReadBytes", limit)) if limit == MAX_BYTES as i64
    ));
    assert_eq!(source.position(), (MAX_BYTES + 1) as u64);

    struct BrokenReader;
    impl Read for BrokenReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "injected read failure",
            ))
        }
    }
    assert!(matches!(
        read_limited(BrokenReader),
        Err(Failure::Io(error)) if error.kind() == io::ErrorKind::PermissionDenied
    ));
}

#[test]
fn invalid_bytes_never_create_or_truncate_a_target_and_report_original_offset() {
    let root = Directory::new();
    let existing = root.path("existing.bin");
    let absent = root.path("absent.bin");
    fs::write(&existing, b"preserved").expect("existing file");
    for value in [-1, 256] {
        for path in [&existing, &absent] {
            let returned = invoke(
                FsIntrinsic::WriteBytes,
                path,
                Some(vec![
                    Value::Integer(0),
                    Value::Integer(255),
                    Value::Integer(value),
                ]),
            );
            assert_error(
                &returned,
                "InvalidByte",
                &[Value::Integer(2), Value::Integer(value)],
            );
        }
        assert_eq!(fs::read(&existing).expect("preserved bytes"), b"preserved");
        assert!(!Path::new(&absent).exists());
        assert!(matches!(
            write_bytes("bad\0path", &[Value::Integer(value)], location()).expect("integer array"),
            Err(Failure::InvalidByte { offset: 0, value: found }) if found == value
        ));
    }
}

#[test]
fn write_size_preflight_wins_over_invalid_bytes_and_target_errors() {
    let root = Directory::new();
    let existing = root.path("existing.bin");
    let absent = root.path("absent.bin");
    fs::write(&existing, b"preserved").expect("existing file");
    let mut values = vec![Value::Integer(0); MAX_BYTES + 1];
    values[0] = Value::Integer(-1);
    for path in [existing.as_str(), absent.as_str(), "bad\0path"] {
        assert!(matches!(
            write_bytes(path, &values, location()).expect("size checked before element access"),
            Err(Failure::LimitExceeded("Std.Fs.WriteBytes", limit)) if limit == MAX_BYTES as i64
        ));
    }
    assert_eq!(fs::read(&existing).expect("preserved bytes"), b"preserved");
    assert!(!Path::new(&absent).exists());
    let returned = super::results::result(
        &crate::intrinsics::TEST_AGGREGATES,
        "Std.Fs.WriteBytes",
        std::slice::from_ref(&existing),
        write_bytes(&existing, &values, location()).expect("size preflight"),
        |()| Value::Boolean(true),
        location(),
    )
    .expect("filesystem error layout");
    assert_error(
        &returned,
        "LimitExceeded",
        &[
            Value::Str("Std.Fs.WriteBytes".into()),
            Value::Integer(MAX_BYTES as i64),
        ],
    );
}

#[test]
fn missing_read_and_write_targets_keep_filesystem_categories_and_host_causes() {
    let root = Directory::new();
    let missing = root.path("missing.bin");
    let below_missing = root.path("absent/child.bin");
    for (operation, name, path, bytes) in [
        (FsIntrinsic::ReadBytes, "Std.Fs.ReadBytes", &missing, None),
        (
            FsIntrinsic::WriteBytes,
            "Std.Fs.WriteBytes",
            &below_missing,
            Some(vec![]),
        ),
    ] {
        let returned = invoke(operation, path, bytes);
        let Value::ResultError(error) = returned else {
            panic!("filesystem failure");
        };
        let Value::Enum(error) = error.as_ref() else {
            panic!("typed filesystem error");
        };
        assert_eq!(error.body().layout.variant, "FileNotFound");
        assert_eq!(error.body().values[0], Value::Str(path.as_str().into()));
        let Value::Enum(cause) = &error.body().values[1] else {
            panic!("host cause");
        };
        assert_eq!(cause.body().layout.type_name, "Std.Errors.HostError");
        assert_eq!(cause.body().values[0], Value::Str(name.into()));
        assert_eq!(
            cause.body().values[1],
            Value::Array(vec![Value::Str(path.as_str().into())].into())
        );
    }
}
