//! Per-VM read-only host collection with task-side typed value construction.
mod collector;
mod data;
mod source;
#[cfg(test)]
mod tests;
pub(in crate::execution::vm) mod wait;

use crate::execution::vm::{VmError, worker::Worker};
use crate::{Intrinsic, SourceLocation, SystemInfoIntrinsic as Op, Value};
use fpas_builtins::{
    host_info::{self, Shape},
    native_errors::NativeErrorFamily,
};
use serde_json::Value as Json;
use std::sync::{Mutex, atomic::AtomicBool};

#[derive(Default)]
pub(super) struct State {
    source: Mutex<Option<source::Source>>,
    busy: AtomicBool,
}

impl State {
    fn submit(
        &self,
        operation: Op,
    ) -> Result<std::sync::mpsc::Receiver<Result<Json, String>>, String> {
        let mut source = self.source.lock().unwrap_or_else(|p| p.into_inner());
        if source.is_none() {
            *source = Some(source::Source::new()?);
        }
        source
            .as_ref()
            .ok_or("Host information collector is unavailable")?
            .submit(operation)
    }
    fn query(&self, operation: Op) -> Result<Json, String> {
        self.submit(operation)?
            .recv()
            .map_err(|_| "Host information collector exited".to_owned())?
    }
}

enum Failure {
    Query(String),
    Range(String),
    Contract(String),
    Vm(VmError),
}

impl Worker {
    pub(super) fn execute_host_info_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::SystemInfo(operation) = intrinsic else {
            return Ok(None);
        };
        self.require_host_info_arguments(arguments)?;
        let result = {
            let _wait = self.native_wait(Vec::new())?;
            self.hosted.host_info.query(operation)
        };
        self.host_info_result(operation, result, location)
            .map(|v| Some(Some(v)))
    }

    pub(in crate::execution::vm) fn require_host_info_arguments(
        &self,
        values: &[Value],
    ) -> Result<(), VmError> {
        if values.is_empty() {
            return Ok(());
        }
        Err(self.runtime_error(
            fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            "Host information queries take no arguments",
            "Call the selected query without arguments.",
        ))
    }

    pub(in crate::execution::vm) fn host_info_result(
        &self,
        operation: Op,
        result: Result<Json, String>,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        let Some((_, shape)) = host_info::QUERIES
            .iter()
            .find(|(name, _)| *name == operation.source_name())
        else {
            return Err(self.runtime_error(
                fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE,
                "Host information query has no declaration",
                "Rebuild compiler and runtime together.",
            ));
        };
        let result = result.map_err(Failure::Query).and_then(|data| {
            self.host_info_value(*shape, &data, operation.source_name(), location)
        });
        match result {
            Ok(value) => Ok(Value::result_ok(value)),
            Err(error) => Ok(Value::result_error(self.host_info_error(
                operation.source_name(),
                error,
                location,
            )?)),
        }
    }

    fn host_info_error(
        &self,
        operation: &str,
        failure: Failure,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        let (variant, fields) = match failure {
            Failure::Vm(error) => return Err(error),
            Failure::Contract(reason) => {
                return Err(self.runtime_error(
                    fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE,
                    reason,
                    "Rebuild compiler and runtime together.",
                ));
            }
            Failure::Query(reason) => (
                0,
                vec![Value::Str(operation.into()), Value::Str(reason.into())],
            ),
            Failure::Range(field) => (1, vec![Value::Str(field.into())]),
        };
        crate::stdlib::native_errors::value(
            self.layouts.as_ref(),
            NativeErrorFamily::SystemInfo,
            variant,
            fields,
            location,
        )
    }

    fn host_info_value(
        &self,
        shape: Shape,
        data: &Json,
        path: &str,
        location: SourceLocation,
    ) -> Result<Value, Failure> {
        let mismatch = || Failure::Contract(format!("Invalid host information field {path}"));
        Ok(match shape {
            Shape::Boolean => Value::Boolean(data.as_bool().ok_or_else(mismatch)?),
            Shape::Integer => Value::Integer(data.as_i64().ok_or_else(|| {
                if data.is_number() {
                    Failure::Range(path.into())
                } else {
                    mismatch()
                }
            })?),
            Shape::Real => Value::Real(
                data.as_f64()
                    .filter(|v| v.is_finite())
                    .ok_or_else(mismatch)?,
            ),
            Shape::String => Value::Str(data.as_str().ok_or_else(mismatch)?.into()),
            Shape::Option(inner) => {
                if data.is_null() {
                    Value::OptionNone
                } else {
                    Value::option_some(self.host_info_value(*inner, data, path, location)?)
                }
            }
            Shape::Array(inner) => Value::Array(
                data.as_array()
                    .ok_or_else(mismatch)?
                    .iter()
                    .map(|data| self.host_info_value(*inner, data, path, location))
                    .collect::<Result<Vec<_>, _>>()?
                    .into(),
            ),
            Shape::Record(name) => {
                let declaration = host_info::record(name).ok_or_else(mismatch)?;
                let values = declaration
                    .fields
                    .iter()
                    .map(|(name, shape)| {
                        let path = format!("{path}.{name}");
                        let value = data.get(*name).ok_or_else(|| {
                            Failure::Contract(format!("Missing host information field {path}"))
                        })?;
                        self.host_info_value(*shape, value, &path, location)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.record_value(name, values, location)
                    .map_err(Failure::Vm)?
            }
            Shape::Result(inner) => {
                let result = match data.get("Ok") {
                    Some(data) => self.host_info_value(*inner, data, path, location),
                    None => Err(Failure::Query(
                        data.get("Error")
                            .and_then(Json::as_str)
                            .ok_or_else(mismatch)?
                            .into(),
                    )),
                };
                match result {
                    Ok(value) => Value::result_ok(value),
                    Err(error) => Value::result_error(
                        self.host_info_error(path, error, location)
                            .map_err(Failure::Vm)?,
                    ),
                }
            }
        })
    }
}
