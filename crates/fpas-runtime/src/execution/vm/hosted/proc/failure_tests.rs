//! Legacy wait errors keep their category while controlled calls retain output.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "typed process boundary assertions"
)]

use std::sync::Arc;
use std::time::Duration;

use crate::{EnumTypeId, EnumVariantId, RuntimeEnumLayout, SharedEnum, SourceLocation, Value};

use super::failure::Kind;
use super::options::{Options, Request};
use super::registry::Processes;

struct Factory;

impl crate::stdlib::AggregateFactory for Factory {
    fn record(
        &self,
        _name: &str,
        _values: Vec<Value>,
        _location: SourceLocation,
    ) -> Result<Value, crate::stdlib::StdError> {
        panic!("this error conversion does not construct records")
    }

    fn enumeration(
        &self,
        name: &str,
        variant: &str,
        values: Vec<Value>,
        _location: SourceLocation,
    ) -> Result<Value, crate::stdlib::StdError> {
        Ok(Value::Enum(SharedEnum::new(
            Arc::new(RuntimeEnumLayout {
                error: None,
                enumeration: EnumTypeId::new(0),
                variant_id: EnumVariantId::new(0),
                type_name: name.into(),
                variant: variant.into(),
                fields: (0..values.len())
                    .map(|index| format!("field{index}"))
                    .collect(),
            }),
            values,
        )))
    }
}

#[test]
fn signalled_legacy_capture_keeps_wait_failed_and_options_keep_partial_execution_failure() {
    for (legacy, operation, expected) in [
        (true, "Std.Proc.RunCapture", "WaitFailed"),
        (false, "Std.Proc.RunCaptureWithOptions", "ExecutionFailed"),
    ] {
        let request = Request {
            command: "/bin/sh",
            arguments: vec!["-c", "printf prefix; kill -TERM $$"],
            options: Options {
                timeout: Some(Duration::from_secs(10)),
                ..Options::default()
            },
            capture: true,
            inherit_input: false,
        };
        let failure = Processes::new()
            .run(&request, None, || false)
            .err()
            .expect("signalled child");
        assert!(matches!(failure.kind, Kind::Wait(_)));
        assert_eq!(failure.stdout, "prefix");
        let value = failure
            .into_value(
                &Factory,
                operation,
                request.command,
                legacy,
                SourceLocation::new(1, 1),
            )
            .expect("declared error");
        let Value::Enum(error) = value else {
            panic!("typed process error");
        };
        assert_eq!(error.body().layout.type_name, "Std.Proc.ProcError");
        assert_eq!(error.body().layout.variant, expected);
        let Value::Enum(cause) = &error.body().values[0] else {
            panic!("host cause");
        };
        assert_eq!(cause.body().layout.type_name, "Std.Errors.HostError");
        assert_eq!(cause.body().values[0], Value::Str(operation.into()));
        if !legacy {
            assert_eq!(error.body().values[1], Value::Str("prefix".into()));
            assert_eq!(error.body().values[2], Value::Str("".into()));
        }
    }
}
