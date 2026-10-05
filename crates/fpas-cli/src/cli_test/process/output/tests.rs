//! Program-stderr capture boundaries preserve complete records and a single truncation event.

use super::ProgramStderrBuffer;
use crate::cli_output::{DiagnosticFormat, Reporter};

#[test]
fn full_record_at_the_limit_survives_and_overflow_stops_further_records() {
    let (capture, receiver) = ProgramStderrBuffer::new(DiagnosticFormat::Json);
    let receiver = receiver.expect("JSON receiver");
    let first = "ä😀";
    let rendered = fpas_diagnostics::render_program_stderr_json(first).expect("JSON");
    capture.buffer.lock().expect("capture lock").limit = rendered.len() + 1;
    receiver(first);
    receiver("next line exceeds the budget");
    receiver("discard this later line too");

    let mut output = Vec::new();
    capture.forward(&mut Reporter::new(DiagnosticFormat::Json, &mut output));
    let output = String::from_utf8(output).expect("UTF-8");
    let records = output
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("complete JSON record"))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["text"], first);
    assert_eq!(records[1]["kind"], "program-output");
    assert_eq!(
        records[1]["text"],
        "Program stderr was truncated at the 8 MiB capture limit."
    );
}

#[test]
fn oversized_first_line_emits_only_the_truncation_event() {
    let (capture, receiver) = ProgramStderrBuffer::new(DiagnosticFormat::Json);
    capture.buffer.lock().expect("capture lock").limit = 0;
    receiver.expect("JSON receiver")("this record cannot fit");
    let mut output = Vec::new();
    capture.forward(&mut Reporter::new(DiagnosticFormat::Json, &mut output));
    let record: serde_json::Value = serde_json::from_slice(&output).expect("one complete record");
    assert_eq!(
        record["text"],
        "Program stderr was truncated at the 8 MiB capture limit."
    );
}
