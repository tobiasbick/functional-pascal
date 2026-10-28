//! Negotiated SDK state becomes FPAS values only on the VM lane.

use crate::execution::vm::worker::Worker;
use crate::stdlib::AggregateFactory;
use crate::{SourceLocation, Value};
use fpas_acp::binding::CatalogCodec as _;
use fpas_acp::{
    binding::{Shape, Value as Detached},
    connection::{Fault, Lifecycle, Snapshot},
};

impl Worker {
    pub(super) fn acp_state(
        &self,
        snapshot: Snapshot,
        location: SourceLocation,
    ) -> Result<Value, Fault> {
        let selected = snapshot
            .negotiated_version
            .unwrap_or(fpas_acp::schema::Version::V1);
        let mut converter = self.acp_converter(selected)?;
        let mut version = Value::OptionNone;
        let mut capabilities = Value::OptionNone;
        let mut info = Value::OptionNone;
        let mut auth = Value::Array(Vec::new().into());
        let mut v2_capabilities = Value::OptionNone;
        let mut v2_info = Value::OptionNone;
        let mut v2_auth = Value::Array(Vec::new().into());
        if let Some(raw) = snapshot.initialization {
            let Detached::Record { fields: values, .. } =
                converter.catalog.decode("InitializeResponse", &raw)?
            else {
                return Err(Fault::InvalidMessage);
            };
            let Shape::Record(fields) = &converter.catalog.declarations["InitializeResponse"].shape
            else {
                return Err(Fault::InvalidMessage);
            };
            for (field, value) in fields.iter().zip(values) {
                match field.name.as_str() {
                    "ProtocolVersion" => {
                        version = Value::option_some(converter.pack(&field.ty, value, None)?)
                    }
                    "AgentCapabilities" => {
                        capabilities = Value::option_some(converter.pack(&field.ty, value, None)?)
                    }
                    "AgentInfo" => info = converter.pack(&field.ty, value, None)?,
                    "AuthMethods" => auth = converter.pack(&field.ty, value, None)?,
                    "Capabilities" => {
                        v2_capabilities =
                            Value::option_some(converter.pack(&field.ty, value, None)?)
                    }
                    "Info" => v2_info = Value::option_some(converter.pack(&field.ty, value, None)?),
                    _ => {}
                }
            }
            if selected == fpas_acp::schema::Version::V2 {
                v2_auth = auth;
                auth = Value::Array(Vec::new().into());
            }
        }
        self.layouts
            .record(
                "Std.Acp.ConnectionState",
                vec![
                    Value::Integer(match snapshot.lifecycle {
                        Lifecycle::Uninitialized => 0,
                        Lifecycle::Initializing => 1,
                        Lifecycle::Ready => 2,
                        Lifecycle::Closing => 3,
                        Lifecycle::Closed => 4,
                        Lifecycle::Failed => 5,
                    }),
                    Value::Integer(
                        if snapshot.requested_version == fpas_acp::schema::Version::V1 {
                            1
                        } else {
                            2
                        },
                    ),
                    version,
                    capabilities,
                    info,
                    auth,
                    v2_capabilities,
                    v2_info,
                    v2_auth,
                ],
                location,
            )
            .map_err(|_| Fault::Host)
    }
}
