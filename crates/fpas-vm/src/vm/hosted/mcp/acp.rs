//! ACP registration conveniences delegate the same MCP registry and serving frames.

use super::{Failure, arguments as args};
use crate::vm::{VmError, worker::Worker};
use fpas_bytecode::{
    AcpIntrinsic as Op, Intrinsic, McpIntrinsic, Register, RootKind, SourceLocation, Value,
};
use fpas_mcp::Fault;

impl Worker {
    pub(in crate::vm) fn execute_acp_tools(
        &self,
        op: Op,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        if let (Op::RegisterTool, [_, _, handler]) = (op, arguments) {
            self.validate_callback(handler, 2)?;
        }
        let result = (|| -> Result<Value, Failure> {
            match (op, arguments) {
                (Op::RegisterTool, [owner, definition, handler]) => {
                    let connection = self
                        .hosted
                        .acp
                        .connection(args::handle(owner)?)
                        .map_err(|_| Fault::Closed)?;
                    let definition = args::definition(definition)?;
                    let roots = self
                        .domain
                        .root_values(RootKind::Native, vec![handler.clone()]);
                    self.hosted
                        .mcp
                        .register_owned(connection.id(), definition, roots)?;
                    if !connection.state().is_ok_and(|state| {
                        !matches!(
                            state,
                            fpas_acp::connection::Lifecycle::Closing
                                | fpas_acp::connection::Lifecycle::Closed
                                | fpas_acp::connection::Lifecycle::Failed
                        )
                    }) {
                        self.hosted.mcp.release_owned(connection.id());
                        return Err(Fault::Closed.into());
                    }
                    Ok(Value::Boolean(true))
                }
                (Op::AttachMcpServer, [owner, server]) => {
                    let connection = self
                        .hosted
                        .acp
                        .connection(args::handle(owner)?)
                        .map_err(|_| Fault::Closed)?;
                    let id = args::handle(server)?;
                    let server = self.hosted.mcp.server(id)?;
                    let attached = connection
                        .attach_tool_server(
                            &format!("fpas_tools_{id}"),
                            server,
                            self.hosted.mcp.runtime()?,
                            false,
                        )
                        .map_err(|_| Fault::Closed)?;
                    Ok(Value::Boolean(attached))
                }
                _ => Err(Fault::InvalidArgument("Arguments").into()),
            }
        })();
        self.mcp_result(result, location)
    }
    pub(in crate::vm) fn prepare_acp_tools(
        &self,
        connection: &fpas_acp::connection::Connection,
        method: &str,
    ) -> Result<(), fpas_acp::connection::Fault> {
        if matches!(
            method,
            "session/new" | "session/load" | "session/resume" | "session/fork"
        ) && let Some(id) = self.hosted.mcp.owned(connection.id())
        {
            let server = self
                .hosted
                .mcp
                .server(id)
                .map_err(|_| fpas_acp::connection::Fault::Closed)?;
            let runtime = self
                .hosted
                .mcp
                .runtime()
                .map_err(|_| fpas_acp::connection::Fault::Host)?;
            connection.attach_tool_server(&format!("fpas_tools_{id}"), server, runtime, true)?;
        }
        Ok(())
    }
    pub(in crate::vm) fn acp_serve_tools(
        &mut self,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let result = (|| -> Result<(u64, Value), Fault> {
            let [connection, cancellation] = arguments else {
                return Err(Fault::InvalidArgument("Arguments"));
            };
            let connection = self
                .hosted
                .acp
                .connection(args::handle(connection)?)
                .map_err(|_| Fault::Closed)?;
            let id = self
                .hosted
                .mcp
                .owned(connection.id())
                .ok_or(Fault::InvalidArgument("Tools"))?;
            Ok((id, cancellation.clone()))
        })();
        match result {
            Ok((id, token)) => self.mcp_wait_intrinsic(
                Intrinsic::Mcp(McpIntrinsic::Serve),
                &[Value::OpaqueHandle(id), token],
                destination,
            ),
            Err(error) => Ok(Some(Some(
                self.mcp_result(Err(error.into()), self.intrinsic_location())?,
            ))),
        }
    }
}
