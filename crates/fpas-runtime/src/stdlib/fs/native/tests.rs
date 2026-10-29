//! Native path results preserve types and own only unpublished temporary resources.

use super::*;
use crate::stdlib::intrinsics::TEST_AGGREGATES;

fn request(operation: Op, values: &[Value]) -> Call {
    Call::prepare_values(
        operation,
        values,
        &TEST_AGGREGATES,
        SourceLocation::new(1, 1),
    )
    .unwrap()
    .unwrap()
}

#[test]
fn temporary_results_close_unpublished_paths_and_publish_completed_results() {
    for operation in [Op::CreateTempFile, Op::CreateTempDir] {
        let outcome = request(operation, &[]).run(|| false);
        let path = match &outcome.outcome {
            Ok(Data::Temporary(temp)) => temp.path.as_ref().unwrap().clone(),
            _ => panic!("temporary result"),
        };
        assert!(path.exists());
        drop(outcome);
        assert!(!path.exists(), "unpublished temporary resource survived");

        let value = request(operation, &[])
            .run(|| false)
            .into_value(&TEST_AGGREGATES, SourceLocation::new(1, 1))
            .unwrap();
        let Value::ResultOk(value) = value else {
            panic!("Result temporary path")
        };
        let Value::Str(path) = value.as_ref() else {
            panic!("temporary path string")
        };
        let path = PathBuf::from(path.as_ref());
        drop(value);
        assert!(
            path.exists(),
            "publishing must transfer cleanup to the caller"
        );
        if operation == Op::CreateTempDir {
            fs::remove_dir(path).unwrap();
        } else {
            fs::remove_file(path).unwrap();
        }
    }
}

#[test]
fn cancellation_before_path_work_does_not_truncate_or_create_outputs() {
    let directory = temp::create_temp_dir().unwrap();
    let path = Path::new(&directory).join("kept");
    fs::write(&path, b"kept").unwrap();
    for operation in [Op::WriteText, Op::WriteTextAtomic, Op::AppendText] {
        let values = [
            Value::Str(path.to_string_lossy().as_ref().into()),
            Value::Str("lost".into()),
        ];
        let outcome = request(operation, &values).run(|| true);
        assert!(matches!(outcome.outcome, Err(Failure::Cancelled)));
        assert_eq!(fs::read(&path).unwrap(), b"kept");
    }
    let absent = Path::new(&directory).join("absent");
    let values = [Value::Str(absent.to_string_lossy().as_ref().into())];
    assert!(matches!(
        request(Op::CreateDir, &values).run(|| true).outcome,
        Err(Failure::Cancelled)
    ));
    assert!(!absent.exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn partial_atomic_write_cancellation_keeps_the_previous_target_and_no_staging_file() {
    let directory = temp::create_temp_dir().unwrap();
    let path = Path::new(&directory).join("kept");
    fs::write(&path, b"previous").unwrap();
    let values = [
        Value::Str(path.to_string_lossy().as_ref().into()),
        Value::Str("x".repeat(32768).into()),
    ];
    let mut probes = 0;
    let outcome = request(Op::WriteTextAtomic, &values).run(|| {
        probes += 1;
        probes >= 4
    });
    assert!(matches!(outcome.outcome, Err(Failure::Cancelled)));
    assert_eq!(fs::read(&path).unwrap(), b"previous");
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn controlled_read_stops_between_native_chunks() {
    let mut reader = std::io::Cursor::new(vec![1; 32768]);
    let mut probes = 0;
    let outcome = super::super::io::read_prefix(&mut reader, 65536, &mut || {
        probes += 1;
        probes > 1
    });
    assert!(matches!(outcome, Err(Failure::Cancelled)));
    assert!((1..32768).contains(&reader.position()));
}
