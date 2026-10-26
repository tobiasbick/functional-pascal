//! Shared MCP signatures; ACP convenience functions reuse these exact types.

use super::super::{define_func, p};
use super::type_registration::{register_opaque_type as opaque, register_record_type as record};
use crate::{
    check::Checker,
    scope::{Symbol, SymbolKind},
    types::{FunctionTy, Ty},
};
use fpas_std::{native_errors::NativeErrorFamily, std_symbols as s};

fn option(ty: Ty) -> Ty {
    Ty::Option(Box::new(ty))
}
fn array(ty: Ty) -> Ty {
    Ty::Array(Box::new(ty))
}
fn alias(checker: &mut Checker, name: &str, ty: Ty) {
    checker.scopes.imports.define_metadata(
        name,
        Symbol {
            constant: None,
            ty,
            mutable: false,
            kind: SymbolKind::Type,
            capability: crate::TransferCapability::Transferable,
        },
    );
}

pub(super) struct Types {
    pub server: Ty,
    pub endpoint: Ty,
    pub raw: Ty,
    pub definition: Ty,
    pub result: Ty,
    pub context: Ty,
    pub handler: Ty,
    pub error: Ty,
    pub limits: Ty,
    pub http: Ty,
    pub access: Ty,
}

pub(super) fn types(checker: &mut Checker) -> Types {
    let server = opaque(checker, "Std.Mcp.Server");
    let endpoint = opaque(checker, "Std.Mcp.Endpoint");
    let error = super::errors::register(checker, NativeErrorFamily::Mcp);
    for variant in NativeErrorFamily::Mcp.variants() {
        checker.scopes.imports.define_metadata(
            &format!("Std.Mcp.McpError.{}", variant.name),
            Symbol {
                constant: None,
                ty: error.clone(),
                mutable: false,
                kind: SymbolKind::EnumVariantConstructor,
                capability: crate::TransferCapability::Transferable,
            },
        );
    }
    let raw = record(
        checker,
        "Std.Mcp.Types.RawJson",
        vec![("Text".into(), Ty::String)],
    );
    let definition = record(
        checker,
        "Std.Mcp.Types.ToolDefinition",
        vec![
            ("Name".into(), Ty::String),
            ("Description".into(), Ty::String),
            ("InputSchema".into(), raw.clone()),
            ("OutputSchema".into(), option(raw.clone())),
            ("Annotations".into(), option(raw.clone())),
            ("Meta".into(), option(raw.clone())),
        ],
    );
    let result = record(
        checker,
        "Std.Mcp.Types.ToolResult",
        vec![("Payload".into(), raw.clone())],
    );
    let context = record(
        checker,
        "Std.Mcp.Types.RequestContext",
        vec![
            ("Server".into(), server.clone()),
            ("AttachmentId".into(), Ty::Integer),
            ("CallId".into(), Ty::Integer),
            ("RequestId".into(), raw.clone()),
            ("Meta".into(), raw.clone()),
            ("ProtocolVersion".into(), option(Ty::String)),
            ("ConnectionId".into(), option(Ty::Integer)),
            ("SessionId".into(), option(Ty::String)),
        ],
    );
    let header = record(
        checker,
        "Std.Mcp.Types.HttpHeader",
        vec![("Name".into(), Ty::String), ("Value".into(), Ty::String)],
    );
    let access = record(
        checker,
        "Std.Mcp.Types.HttpAccess",
        vec![
            ("Url".into(), Ty::String),
            ("Headers".into(), array(header.clone())),
        ],
    );
    let handler = Ty::Function(FunctionTy {
        type_params: vec![],
        params: vec![p("Context", context.clone()), p("Arguments", raw.clone())],
        return_type: Box::new(Ty::Result(
            Box::new(result.clone()),
            Box::new(error.clone()),
        )),
        variadic: false,
    });
    for (name, ty) in [
        ("RawJson", raw.clone()),
        ("ToolDefinition", definition.clone()),
        ("ToolResult", result.clone()),
        ("RequestContext", context.clone()),
        ("HttpHeader", header),
        ("HttpAccess", access.clone()),
        ("ToolHandler", handler.clone()),
        ("McpError", error.clone()),
    ] {
        alias(checker, &format!("Std.Mcp.{name}"), ty.clone());
        if name == "ToolHandler" {
            alias(checker, "Std.Mcp.Types.ToolHandler", ty);
        }
    }
    let limits = record(
        checker,
        "Std.Mcp.Limits",
        [
            "Tools",
            "CatalogBytes",
            "MessageBytes",
            "PendingCalls",
            "PendingBytes",
            "Endpoints",
        ]
        .into_iter()
        .map(|name| (name.into(), Ty::Integer))
        .collect(),
    );
    let http = record(
        checker,
        "Std.Mcp.HttpOptions",
        vec![
            ("Address".into(), Ty::String),
            ("Connections".into(), Ty::Integer),
            ("RequestMillis".into(), Ty::Integer),
            ("ShutdownMillis".into(), Ty::Integer),
        ],
    );
    Types {
        server,
        endpoint,
        raw,
        definition,
        result,
        context,
        handler,
        error,
        limits,
        http,
        access,
    }
}

pub(super) fn register(checker: &mut Checker) {
    let t = types(checker);
    let result = |value| Ty::Result(Box::new(value), Box::new(t.error.clone()));
    let token = record(checker, "Std.Tasks.CancellationToken", vec![]);
    define_func(checker, s::STD_MCP_DEFAULT_LIMITS, vec![], t.limits.clone());
    define_func(
        checker,
        s::STD_MCP_DEFAULT_HTTP_OPTIONS,
        vec![],
        t.http.clone(),
    );
    define_func(
        checker,
        s::STD_MCP_CREATE,
        vec![p("Limits", t.limits)],
        result(t.server.clone()),
    );
    define_func(
        checker,
        s::STD_MCP_REGISTER_TOOL,
        vec![
            p("Server", t.server.clone()),
            p("Definition", t.definition),
            p("Handler", t.handler),
        ],
        result(Ty::Boolean),
    );
    define_func(
        checker,
        s::STD_MCP_PUBLISH,
        vec![p("Server", t.server.clone()), p("Options", t.http)],
        result(t.endpoint.clone()),
    );
    define_func(
        checker,
        s::STD_MCP_ACCESS,
        vec![p("Endpoint", t.endpoint.clone())],
        result(t.access),
    );
    define_func(
        checker,
        s::STD_MCP_CLOSE_ENDPOINT,
        vec![p("Endpoint", t.endpoint)],
        result(Ty::Boolean),
    );
    for name in [s::STD_MCP_CLOSE, s::STD_MCP_RELEASE] {
        define_func(
            checker,
            name,
            vec![p("Server", t.server.clone())],
            result(Ty::Boolean),
        );
    }
    define_func(
        checker,
        s::STD_MCP_PARSE_JSON,
        vec![p("Text", Ty::String)],
        result(t.raw.clone()),
    );
    define_func(
        checker,
        s::STD_MCP_JSON_MEMBER,
        vec![p("Value", t.raw.clone()), p("Name", Ty::String)],
        result(option(t.raw.clone())),
    );
    define_func(
        checker,
        s::STD_MCP_PARSE_TOOL_RESULT,
        vec![p("Value", t.raw.clone())],
        result(t.result.clone()),
    );
    for name in [s::STD_MCP_TEXT_RESULT, s::STD_MCP_ERROR_RESULT] {
        define_func(checker, name, vec![p("Text", Ty::String)], t.result.clone());
    }
    define_func(
        checker,
        s::STD_MCP_STRUCTURED_RESULT,
        vec![p("Value", t.raw)],
        result(t.result),
    );
    define_func(
        checker,
        s::STD_MCP_IS_CANCELLED,
        vec![p("Context", t.context)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_MCP_SERVE_ONE,
        vec![
            p("Server", t.server.clone()),
            p("TimeoutMillis", option(Ty::Integer)),
            p("Cancellation", option(token.clone())),
        ],
        result(Ty::Boolean),
    );
    define_func(
        checker,
        s::STD_MCP_SERVE,
        vec![p("Server", t.server), p("Cancellation", option(token))],
        result(Ty::Boolean),
    );
}
