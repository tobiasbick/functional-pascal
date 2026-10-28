//! ACP conveniences share nominal MCP definitions and callback ownership.

use super::super::super::{define_func, p};
use super::super::mcp;
use crate::{
    check::Checker,
    scope::{Symbol, SymbolKind},
    types::Ty,
};

pub(super) fn register(checker: &mut Checker) {
    let t = mcp::types(checker);
    let connection = checker
        .scopes
        .imports
        .lookup_metadata("Std.Acp.Connection")
        .map(|symbol| symbol.ty.clone())
        .unwrap_or_else(|| Ty::Named("Std.Acp.Connection".into()));
    for (name, ty) in [
        ("ToolDefinition", t.definition.clone()),
        ("ToolHandler", t.handler.clone()),
        ("ToolResult", t.result),
        ("ToolContext", t.context),
        ("ToolJson", t.raw),
        ("McpError", t.error.clone()),
    ] {
        checker.scopes.imports.define_metadata(
            &format!("Std.Acp.{name}"),
            Symbol {
                constant: None,
                ty,
                mutable: false,
                kind: SymbolKind::Type,
                capability: crate::TransferCapability::Transferable,
            },
        );
    }
    let result = Ty::Result(Box::new(Ty::Boolean), Box::new(t.error));
    if let Ty::Result(_, error) = &result {
        for variant in fpas_builtins::native_errors::NativeErrorFamily::Mcp.variants() {
            checker.scopes.imports.define_metadata(
                &format!("Std.Acp.McpError.{}", variant.name),
                Symbol {
                    constant: None,
                    ty: (**error).clone(),
                    mutable: false,
                    kind: SymbolKind::EnumVariantConstructor,
                    capability: crate::TransferCapability::Transferable,
                },
            );
        }
    }
    define_func(
        checker,
        "Std.Acp.RegisterTool",
        vec![
            p("Connection", connection.clone()),
            p("Definition", t.definition),
            p("Handler", t.handler),
        ],
        result.clone(),
    );
    for name in ["Std.Acp.AttachMcpServer", "Std.Acp.DetachMcpServer"] {
        define_func(
            checker,
            name,
            vec![
                p("Connection", connection.clone()),
                p("Server", t.server.clone()),
            ],
            result.clone(),
        );
    }
    let token = super::super::type_registration::register_record_type(
        checker,
        "Std.Tasks.CancellationToken",
        vec![],
    );
    define_func(
        checker,
        "Std.Acp.ServeTools",
        vec![
            p("Connection", connection),
            p("Cancellation", Ty::Option(Box::new(token))),
        ],
        result,
    );
}
