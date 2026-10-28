//! Complete graph checks before host arguments enter a worker.

use crate::{GraphClassification, InstructionAddress, Value, VerifiedExecutable, inspect_values};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use super::{HostArgument, HostValueError, HostValues};
use crate::execution::vm::{VmError, diagnostics};

pub(super) fn inspect(value: &Value) -> Result<(GraphClassification, bool), HostValueError> {
    let mut resources = false;
    let graph = inspect_values(std::slice::from_ref(value), |value| {
        resources |= match value {
            Value::OpaqueHandle(_) | Value::Task(_) => true,
            Value::Record(record) => {
                let name = &record.body().layout.type_name;
                name.eq_ignore_ascii_case("Std.Net.Http.Handles.BodyStream")
                    || name.eq_ignore_ascii_case("Std.Net.Http.Handles.SseDecoder")
            }
            _ => false,
        };
    })
    .map_err(HostValueError::Ownership)?;
    Ok((graph, resources))
}

impl HostValues {
    pub(in crate::execution::vm) fn admit(
        &self,
        arguments: Vec<HostArgument>,
        executable: &VerifiedExecutable,
    ) -> Result<(Vec<Value>, Vec<crate::RootValues>), VmError> {
        // Validate the entire batch before taking any value or starting the target. Dropping
        // rejected arguments uses their iterative teardown even for very deep wrapper chains.
        for argument in &arguments {
            self.check(argument, executable).map_err(|message| {
                diagnostics::at_address(
                    executable.executable(),
                    InstructionAddress::new(0),
                    RUNTIME_INVALID_TASK,
                    message,
                    "Return session resources and mutable closures only to their originating session; share the same executable image for callable data.",
                )
            })?;
        }
        let mut values = Vec::with_capacity(arguments.len());
        let mut roots = Vec::new();
        for mut argument in arguments {
            let value = match &mut argument {
                HostArgument::Data(value) => std::mem::replace(value, Value::Unit),
                HostArgument::Export(value) => {
                    let _access = value.0.origin.domain.access();
                    let copy = value.0.value.clone();
                    if std::sync::Arc::ptr_eq(&self.origin, &value.0.origin) {
                        // Register before consuming the final source handle. Foreign inputs
                        // are cell-free and never acquire a target lock under a source lock.
                        roots.push(
                            self.origin
                                .domain
                                .root_values(crate::RootKind::Native, vec![copy.clone()]),
                        );
                    }
                    copy
                }
            };
            values.push(value);
        }
        Ok((values, roots))
    }

    fn check(
        &self,
        argument: &HostArgument,
        executable: &VerifiedExecutable,
    ) -> Result<(), String> {
        let _source_access = match argument {
            HostArgument::Export(value) => Some(value.0.origin.domain.access()),
            HostArgument::Data(_) => None,
        };
        let (value, same_session) = match argument {
            HostArgument::Data(value) => (value, false),
            HostArgument::Export(value) => (
                &value.0.value,
                std::sync::Arc::ptr_eq(&self.origin, &value.0.origin),
            ),
        };
        let (graph, resources) = inspect(value).map_err(|error| error.to_string())?;
        if graph.has_references {
            return Err("A variable reference cannot cross a host boundary".into());
        }
        if graph
            .executable_identities
            .iter()
            .any(|identity| identity != executable.identity())
        {
            return Err("Callable value belongs to a different executable image".into());
        }
        if same_session {
            graph
                .validate_domain(&self.origin.domain)
                .map_err(|error| error.to_string())
        } else if !graph.is_transferable() || resources {
            Err("Value requires its originating task or session".into())
        } else {
            Ok(())
        }
    }
}
