use std::io::Write;

use super::{CappedBuffer, run_worker_from_args};

#[test]
fn capped_worker_output_reports_overflow() {
    let mut output = CappedBuffer::new(4);

    assert!(output.write_all(b"pass").is_ok());
    assert!(output.write_all(b"!").is_err());
    assert!(output.overflowed());
    assert_eq!(output.into_inner(), b"pass");
}

#[test]
fn public_arguments_do_not_enter_worker_mode() {
    assert_eq!(run_worker_from_args(&["test".to_string()]), None);
}
