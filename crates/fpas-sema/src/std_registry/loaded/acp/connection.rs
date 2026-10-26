//! Connection signatures and detached lifetime data, separate from SDK payload shapes.

use super::super::{
    super::{define_func, p},
    type_registration::{register_enum_type as simple_enum, register_record_type as record},
};
use crate::{
    check::Checker,
    scope::{Symbol, SymbolKind},
    types::{EnumTy, EnumVariantTy, Ty},
};
use std::sync::Arc;

fn named(checker: &Checker, name: &str) -> Ty {
    checker
        .scopes
        .imports
        .lookup_metadata(name)
        .unwrap_or_else(|| panic!("ACP prerequisite {name} is not registered"))
        .ty
        .clone()
}
fn option(ty: Ty) -> Ty {
    Ty::Option(Box::new(ty))
}
fn array(ty: Ty) -> Ty {
    Ty::Array(Box::new(ty))
}

pub(super) fn register(checker: &mut Checker) {
    let connection = named(checker, "Std.Acp.Connection");
    let responder = named(checker, "Std.Acp.Responder");
    let token = named(checker, "Std.Acp.RequestToken");
    let endpoint = named(checker, "Std.Acp.MessageEndpoint");
    let raw = named(checker, "Std.Acp.Types.RawJson");
    let error = named(checker, "Std.Acp.Types.AcpError");
    let protocol_error = named(checker, "Std.Acp.Types.ProtocolError");
    let version = simple_enum(checker, "Std.Acp.ProtocolVersion", &["V1", "V2"]);
    let lifecycle = simple_enum(
        checker,
        "Std.Acp.Lifecycle",
        &[
            "Uninitialized",
            "Initializing",
            "Ready",
            "Closing",
            "Closed",
            "Failed",
        ],
    );
    let limits = record(
        checker,
        "Std.Acp.Limits",
        [
            "OutgoingRequests",
            "IncomingRequests",
            "Sessions",
            "Terminals",
            "Routes",
            "QueueEntries",
            "OutboundEntries",
            "QueueBytes",
            "MessageBytes",
            "StderrBytes",
            "ShutdownMillis",
            "TerminalBytes",
            "AllTerminalBytes",
        ]
        .into_iter()
        .map(|name| (name.into(), Ty::Integer))
        .collect(),
    );
    let experimental = record(
        checker,
        "Std.Acp.ExperimentalOptions",
        [
            "TokenUsage",
            "Providers",
            "McpOverAcp",
            "PlanOperations",
            "Fork",
            "SystemPrompt",
            "Subagents",
        ]
        .into_iter()
        .map(|name| (name.into(), Ty::Boolean))
        .collect(),
    );
    let options = record(
        checker,
        "Std.Acp.Options",
        vec![
            (
                "ClientInfo".into(),
                option(named(checker, "Std.Acp.Types.ImplementationValue")),
            ),
            (
                "ClientCapabilities".into(),
                named(checker, "Std.Acp.Types.ClientCapabilities"),
            ),
            ("SupportedVersions".into(), array(version)),
            ("ExperimentalOptions".into(), experimental),
            ("Limits".into(), limits),
            (
                "V2ClientCapabilities".into(),
                named(checker, "Std.Acp.V2.ClientCapabilities"),
            ),
        ],
    );
    let environment = record(
        checker,
        "Std.Acp.EnvironmentVariable",
        vec![
            ("Name".into(), Ty::String),
            ("Value".into(), option(Ty::String)),
        ],
    );
    let launch = record(
        checker,
        "Std.Acp.Launch",
        vec![
            ("Command".into(), Ty::String),
            ("Arguments".into(), array(Ty::String)),
            ("WorkingDirectory".into(), option(Ty::String)),
            ("Environment".into(), array(environment)),
        ],
    );
    let messages = record(
        checker,
        "Std.Acp.MessageConnection",
        vec![
            ("Connection".into(), connection.clone()),
            ("Endpoint".into(), endpoint.clone()),
        ],
    );
    let process_status = enumeration(
        checker,
        "ProcessExitStatus",
        vec![
            ("Exited", vec![("Code", Ty::Integer)]),
            ("Signaled", vec![("Signal", Ty::Integer)]),
            ("Unknown", vec![]),
        ],
    );
    let closed = record(
        checker,
        "Std.Acp.ClosedStatus",
        vec![
            ("Cause".into(), option(error.clone())),
            ("ProcessExit".into(), option(process_status)),
            ("Stderr".into(), Ty::String),
            ("StderrTruncated".into(), Ty::Boolean),
        ],
    );
    let state = record(
        checker,
        "Std.Acp.ConnectionState",
        vec![
            ("Lifecycle".into(), lifecycle),
            ("RequestedVersion".into(), Ty::Integer),
            ("NegotiatedVersion".into(), option(Ty::Integer)),
            (
                "AgentCapabilities".into(),
                option(named(checker, "Std.Acp.Types.AgentCapabilities")),
            ),
            (
                "AgentInfo".into(),
                option(named(checker, "Std.Acp.Types.ImplementationValue")),
            ),
            (
                "AuthMethods".into(),
                array(named(checker, "Std.Acp.Types.AuthMethod")),
            ),
            (
                "V2AgentCapabilities".into(),
                option(named(checker, "Std.Acp.V2.AgentCapabilities")),
            ),
            (
                "V2AgentInfo".into(),
                option(named(checker, "Std.Acp.V2.ImplementationValue")),
            ),
            (
                "V2AuthMethods".into(),
                array(named(checker, "Std.Acp.V2.AuthMethod")),
            ),
        ],
    );
    let mut variants = vec![
        (
            "McpResponse",
            vec![
                ("Token", token.clone()),
                (
                    "Payload",
                    named(checker, "Std.Acp.Experimental.MessageMcpResponse"),
                ),
            ],
        ),
        (
            "V2McpResponse",
            vec![
                ("Token", token.clone()),
                ("Payload", named(checker, "Std.Acp.V2.MessageMcpResponse")),
            ],
        ),
        (
            "RequestFailed",
            vec![("Token", token.clone()), ("Cause", error.clone())],
        ),
        ("RequestCancelled", vec![("Responder", responder.clone())]),
        (
            "UnknownNotification",
            vec![("Method", Ty::String), ("Payload", raw.clone())],
        ),
        ("Closed", vec![("Status", closed)]),
    ];
    for (module, prefix) in [("Std.Acp.Types", ""), ("Std.Acp.V2", "V2")] {
        for (variant, root, fields) in [
            ("Response", "AgentResponse", vec![("Token", token.clone())]),
            ("Notification", "AgentNotification", vec![]),
            ("Request", "AgentRequest", vec![]),
        ] {
            let mut fields = fields;
            fields.push(("Payload", named(checker, &format!("{module}.{root}"))));
            if variant == "Request" {
                fields.push(("Responder", responder.clone()));
            }
            let name = match (prefix, variant) {
                ("", "Response") => "Response",
                ("", "Notification") => "Notification",
                ("", _) => "Request",
                (_, "Response") => "V2Response",
                (_, "Notification") => "V2Notification",
                _ => "V2Request",
            };
            variants.push((name, fields));
        }
    }
    variants.push((
        "V2Initialized",
        vec![
            ("Token", token.clone()),
            ("Outcome", named(checker, "Std.Acp.V2.InitializeOutcome")),
        ],
    ));
    let payload = enumeration(checker, "EventPayload", variants);
    let event = record(
        checker,
        "Std.Acp.Event",
        vec![
            ("Payload".into(), payload),
            ("RawMessage".into(), option(raw.clone())),
        ],
    );
    let cancellation = if let Some(symbol) = checker
        .scopes
        .imports
        .lookup_metadata("Std.Tasks.CancellationToken")
    {
        symbol.ty.clone()
    } else {
        record(checker, "Std.Tasks.CancellationToken", vec![])
    };
    define_func(checker, "Std.Acp.DefaultOptions", vec![], options.clone());
    define_func(
        checker,
        "Std.Acp.RegisterMcpEndpoint",
        vec![
            p("Connection", connection.clone()),
            p("ServerId", Ty::String),
            p("SessionId", option(Ty::String)),
        ],
        Ty::Result(
            Box::new(named(checker, "Std.Acp.McpEndpoint")),
            Box::new(error.clone()),
        ),
    );
    define_func(
        checker,
        "Std.Acp.RemoveMcpEndpoint",
        vec![
            p("Connection", connection.clone()),
            p("Endpoint", named(checker, "Std.Acp.McpEndpoint")),
        ],
        Ty::Result(Box::new(Ty::Boolean), Box::new(error.clone())),
    );
    for (name, parameters, output) in [
        (
            "ConnectMessages",
            vec![("Options", options.clone())],
            messages,
        ),
        (
            "ConnectProcess",
            vec![("Launch", launch), ("Options", options)],
            connection.clone(),
        ),
        (
            "FeedMessage",
            vec![("Endpoint", endpoint.clone()), ("Message", raw.clone())],
            Ty::Boolean,
        ),
        (
            "TryTakeMessage",
            vec![("Endpoint", endpoint.clone())],
            option(raw),
        ),
        ("CloseInput", vec![("Endpoint", endpoint)], Ty::Boolean),
        (
            "TryReceive",
            vec![("Connection", connection.clone())],
            option(event.clone()),
        ),
        (
            "Receive",
            vec![
                ("Connection", connection.clone()),
                ("TimeoutMillis", option(Ty::Integer)),
                ("CancellationToken", option(cancellation)),
            ],
            event,
        ),
        (
            "RespondError",
            vec![("Responder", responder.clone()), ("Cause", protocol_error)],
            Ty::Boolean,
        ),
        (
            "CancelRequest",
            vec![("Connection", connection.clone()), ("Token", token.clone())],
            Ty::Boolean,
        ),
        ("State", vec![("Connection", connection.clone())], state),
        (
            "Close",
            vec![("Connection", connection.clone())],
            Ty::Boolean,
        ),
        (
            "Release",
            vec![("Connection", connection.clone())],
            Ty::Boolean,
        ),
    ] {
        define_func(
            checker,
            &format!("Std.Acp.{name}"),
            parameters
                .into_iter()
                .map(|(name, ty)| p(name, ty))
                .collect(),
            Ty::Result(Box::new(output), Box::new(error.clone())),
        );
    }
    for module in ["Std.Acp.Types", "Std.Acp.V2"] {
        let unit = if module == "Std.Acp.Types" {
            "Std.Acp"
        } else {
            module
        };
        let payload = named(
            checker,
            if module == "Std.Acp.Types" {
                "Std.Acp.Experimental.MessageMcpRequest"
            } else {
                "Std.Acp.V2.MessageMcpRequest"
            },
        );
        define_func(
            checker,
            &format!("{unit}.SendMcp"),
            vec![
                p("Endpoint", named(checker, "Std.Acp.McpEndpoint")),
                p("Payload", payload),
            ],
            Ty::Result(Box::new(token.clone()), Box::new(error.clone())),
        );
        for (operation, input, receiver, output) in [
            ("Send", "ClientRequest", connection.clone(), token.clone()),
            (
                "Notify",
                "ClientNotification",
                connection.clone(),
                Ty::Boolean,
            ),
            ("Respond", "ClientResponse", responder.clone(), Ty::Boolean),
        ] {
            let input = named(checker, &format!("{module}.{input}"));
            define_func(
                checker,
                &format!("{unit}.{operation}"),
                vec![p("Owner", receiver), p("Payload", input)],
                Ty::Result(Box::new(output), Box::new(error.clone())),
            );
        }
    }
}

fn enumeration(checker: &mut Checker, short: &str, variants: Vec<(&str, Vec<(&str, Ty)>)>) -> Ty {
    let name = format!("Std.Acp.{short}");
    let variants = variants
        .into_iter()
        .map(|(name, fields)| EnumVariantTy {
            name: name.into(),
            fields: fields
                .into_iter()
                .map(|(name, ty)| (name.into(), ty))
                .collect(),
            message: None,
            backing_value: None,
        })
        .collect::<Vec<_>>();
    let ty = Ty::Enum(Arc::new(EnumTy {
        name: name.clone(),
        is_error: false,
        type_params: vec![],
        type_args: vec![],
        variants: variants.clone(),
    }));
    for (name, kind) in
        std::iter::once((name.clone(), SymbolKind::Type)).chain(variants.iter().map(|variant| {
            (
                format!("{name}.{}", variant.name),
                if variant.fields.is_empty() {
                    SymbolKind::EnumMember
                } else {
                    SymbolKind::EnumVariantConstructor
                },
            )
        }))
    {
        checker.scopes.imports.define_metadata(
            &name,
            Symbol {
                ty: ty.clone(),
                constant: None,
                kind,
                mutable: false,
                capability: crate::TransferCapability::Transferable,
            },
        );
    }
    ty
}
